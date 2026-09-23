use std::sync::Arc;

use my_http_server::{
    async_trait, hyper::Method, HttpContext, HttpFailResult, HttpOkResult, HttpServerMiddleware,
};

use crate::app::AppContext;

/// The header the SDK writer sends with every request it makes (since my-no-sql-sdk 0.4.1 of
/// June 2026). The node's readers send it only to `/api/DataReader/*`, the UI never.
const WRITER_SESSION_HEADER: &str = "session";

/// Every route of the main node's api a writer may call, lowercased, with and without `/api`:
/// the writes, the reads, the writer's ping, and `/Rows` - the route the SDK's
/// `delete_partitions` calls. Nothing else of the main node is reachable through the node - no
/// administration, and no `Tables/MigrateFrom`, which would make the main node fetch any url of
/// its datacenter for whoever reaches the node.
const WRITER_ROUTES: [&str; 76] = [
    // writes
    "/api/bulk/cleanandbulkinsert",
    "/api/bulk/cleanandbulkinsertbychunks",
    "/api/bulk/cleanandbulkinsertbychunkscancel",
    "/api/bulk/cleanandbulkinsertbychunkscommit",
    "/api/bulk/delete",
    "/api/bulk/deleteif",
    "/api/bulk/insertorreplace",
    "/api/bulk/insertorreplaceifnew",
    "/api/bulk/insertorreplaceifnewbychunks",
    "/api/bulk/insertorreplaceifnewbychunkscancel",
    "/api/bulk/insertorreplaceifnewbychunkscommit",
    "/api/garbagecollector/cleanandkeepmaxpartitions",
    "/api/garbagecollector/cleanandkeepmaxrecords",
    "/api/ping",
    "/api/row",
    "/api/row/deleteif",
    "/api/row/insert",
    "/api/row/insertorreplace",
    "/api/row/insertorreplaceifnew",
    "/api/row/replace",
    "/api/rows/deletepartitions",
    "/api/tables/clean",
    "/api/tables/create",
    "/api/tables/createifnotexists",
    "/api/tables/delete",
    "/api/tables/updatecompressed",
    "/api/tables/updatepersist",
    "/api/transactions/append",
    "/api/transactions/cancel",
    "/api/transactions/commit",
    "/api/transactions/start",
    "/bulk/cleanandbulkinsert",
    "/bulk/delete",
    "/bulk/insertorreplace",
    "/bulk/insertorreplaceifnew",
    "/garbagecollector/cleanandkeepmaxpartitions",
    "/garbagecollector/cleanandkeepmaxrecords",
    "/row",
    "/row/insert",
    "/row/insertorreplace",
    "/row/insertorreplaceifnew",
    "/row/replace",
    "/rows",
    "/rows/deletepartitions",
    "/tables/clean",
    "/tables/create",
    "/tables/createifnotexists",
    "/tables/delete",
    "/tables/updatecompressed",
    "/tables/updatepersist",
    "/transactions/append",
    "/transactions/cancel",
    "/transactions/commit",
    "/transactions/start",
    // reads
    "/api/count",
    "/count",
    "/api/partitions",
    "/partitions",
    "/api/partitions/count",
    "/partitions/count",
    "/api/partitions/details",
    "/api/row/download",
    "/api/rows/highestrowandbelow",
    "/rows/highestrowandbelow",
    "/api/rows/singlepartitionmultiplerows",
    "/rows/singlepartitionmultiplerows",
    "/api/tables/list",
    "/tables/list",
    "/api/tables/partitionscount",
    "/tables/partitionscount",
    "/api/tables/tablesize",
    "/tables/tablesize",
    "/api/multipart/first",
    "/multipart/first",
    "/api/multipart/next",
    "/multipart/next",
];

/// Reads of the main node's api the node does not serve itself - forwarded whoever asks.
const NOT_SERVED_BY_THE_NODE: [&str; 5] = [
    "/api/multipart/first",
    "/multipart/first",
    "/api/multipart/next",
    "/multipart/next",
    "/api/row/download",
];

/// Reads of one table the node serves from its replica.
const TABLE_READS: [&str; 16] = [
    "/api/row",
    "/row",
    "/api/count",
    "/count",
    "/api/partitions",
    "/partitions",
    "/api/partitions/count",
    "/partitions/count",
    "/api/rows/highestrowandbelow",
    "/rows/highestrowandbelow",
    "/api/rows/singlepartitionmultiplerows",
    "/rows/singlepartitionmultiplerows",
    "/api/tables/partitionscount",
    "/tables/partitionscount",
    "/api/tables/tablesize",
    "/tables/tablesize",
];

/// Decides which requests go to the main node before the node's own actions see them.
///
/// * A writer's request - it carries the `session` header - goes to the main node whole, its
///   reads too: a writer reads what it is about to update, counts rows, lists partitions, and has
///   to see exactly what it would see on the main node, not the replica.
/// * A read of a table the node does not replicate goes to the main node too: whoever asks for it
///   is not one of the node's readers - most likely a writer of an older SDK, which sends no
///   `session` header. It would get `TableNotFound` from the replica.
/// * A read the node does not serve at all (multipart, download) goes to the main node.
///
/// Everything else takes the usual way: reads from the replica, writes through the write actions.
pub struct WriterRequestsMiddleware {
    app: Arc<AppContext>,
}

impl WriterRequestsMiddleware {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }

    fn is_writer(ctx: &HttpContext) -> bool {
        ctx.request
            .data
            .headers()
            .get(WRITER_SESSION_HEADER)
            .and_then(|value| value.to_str().ok())
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false)
    }

    /// The node holds the table the request names, in the namespace it names. A request naming
    /// no table, or a namespace which is not valid, is the node's own to answer.
    fn holds_requested_table(&self, ctx: &HttpContext) -> bool {
        let Some(table_name) = ctx
            .request
            .get_uri()
            .query()
            .and_then(|query| crate::http::find_query_param(query, "tableName"))
        else {
            return true;
        };

        let Ok(table_name) =
            my_http_utils::url_decoder::decode_from_url_query_string(table_name)
        else {
            return true;
        };

        let Ok(namespace) =
            crate::http::parse_namespace_name(crate::http::get_request_namespace_name(ctx))
        else {
            return true;
        };

        match self.app.namespaces.get(namespace) {
            Some(namespace) => namespace.db.get_table(table_name.as_str()).is_some(),
            None => false,
        }
    }
}

#[async_trait::async_trait]
impl HttpServerMiddleware for WriterRequestsMiddleware {
    async fn handle_request(
        &self,
        ctx: &mut HttpContext,
    ) -> Option<Result<HttpOkResult, HttpFailResult>> {
        // Not configured - the write actions answer that writes are refused, reads stay local.
        self.app.main_server_http.as_ref()?;

        let route = normalize_route(ctx.request.http_path.as_str());

        let forward = if Self::is_writer(ctx) {
            is_one_of(&WRITER_ROUTES, route.as_str())
        } else if is_one_of(&NOT_SERVED_BY_THE_NODE, route.as_str()) {
            true
        } else {
            let is_read = ctx.request.method == Method::GET
                || route.ends_with("/singlepartitionmultiplerows");

            is_read
                && is_one_of(&TABLE_READS, route.as_str())
                && !self.holds_requested_table(ctx)
        };

        if !forward {
            return None;
        }

        Some(crate::main_server_http::forward(&self.app, ctx).await)
    }
}

/// Lowercased, no trailing `/` - routes match case-insensitively on the main node too.
fn normalize_route(path: &str) -> String {
    let path = path.trim_end_matches('/');

    if path.is_empty() {
        return "/".to_string();
    }

    path.to_ascii_lowercase()
}

fn is_one_of(routes: &[&str], route: &str) -> bool {
    routes.contains(&route)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_writer_route(path: &str) -> bool {
        is_one_of(&WRITER_ROUTES, normalize_route(path).as_str())
    }

    #[test]
    fn test_writer_routes_go_to_the_main_node() {
        for path in [
            "/Row",
            "/Row/Insert",
            "/api/Row",
            "/api/row/InsertOrReplace/",
            "/Rows",
            "/api/Rows/DeletePartitions",
            "/api/Count",
            "/api/Partitions",
            "/Tables/CreateIfNotExists",
            "/api/Bulk/CleanAndBulkInsert",
            "/api/Transactions/Start",
            "/api/GarbageCollector/CleanAndKeepMaxRecords",
            "/api/Multipart/First",
            "/api/ping",
        ] {
            assert!(is_writer_route(path), "{path}");
        }
    }

    #[test]
    fn test_administration_and_node_routes_are_never_forwarded() {
        for path in [
            "/",
            "/api/Tables/MigrateFrom",
            "/Tables/MigrateFrom",
            "/api/Backup/MakeBackup",
            "/api/Backup/RestoreFromZip",
            "/api/Persist/Force",
            "/api/Settings",
            "/api/Settings/UiWrites",
            "/api/Namespaces",
            "/api/Row/../Backup/MakeBackup",
            "/api/Tables/Anything",
            "/api/DataReader/Subscribe",
            "/api/Status",
            "/api/Connections",
            "/api/IsAlive",
            "/metrics",
            "/data/e2e-items",
            "/assets/app.css",
            "/ping",
        ] {
            assert!(!is_writer_route(path), "{path}");
        }
    }

    #[test]
    fn test_route_lists_are_consistent() {
        for route in NOT_SERVED_BY_THE_NODE.iter().chain(TABLE_READS.iter()) {
            assert!(WRITER_ROUTES.contains(route), "{route}");
        }

        for route in WRITER_ROUTES {
            assert_eq!(route, normalize_route(route), "{route}");
        }
    }
}
