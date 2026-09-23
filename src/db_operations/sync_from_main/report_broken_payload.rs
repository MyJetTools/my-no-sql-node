use my_logger::LogEventCtx;
use my_no_sql_sdk::core::db_json_entity::DbEntityParseFail;

use crate::namespaces::NodeNamespace;

/// A payload the node can not parse is skipped - panicking would only break the connection to
/// the main node, get the very same payload again after the reconnect and break it once more.
pub(super) fn report_broken_payload(
    packet: &str,
    namespace: &NodeNamespace,
    table_name: &str,
    err: &DbEntityParseFail,
) {
    my_logger::LOGGER.write_error(
        "SyncFromMainNode",
        format!("Can not parse {} payload. Err: {:?}", packet, err),
        LogEventCtx::new()
            .add("namespace", namespace.name.to_string())
            .add("tableName", table_name.to_string()),
    );
}
