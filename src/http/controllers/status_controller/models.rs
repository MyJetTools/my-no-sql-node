use my_no_sql_sdk::core::rust_extensions::date_time::DateTimeAsMicroseconds;
use rest_api_shared::{
    InitializedModel, LocationModel, NamespaceStatusModel, ReaderModel, StatusBarModel,
    StatusModel, TableModel, TABLE_SYNC_NOT_FOUND, TABLE_SYNC_PENDING, TABLE_SYNC_REPLICATED,
};

use crate::{
    app::AppContext,
    data_readers::DataReaderConnection,
    namespaces::{NodeNamespace, TableRequestState},
};

pub fn build_status_model(app: &AppContext) -> StatusModel {
    let readers = get_readers(app);

    let mut tables = Vec::new();
    let mut namespaces = Vec::new();

    for namespace in app.namespaces.get_all().iter() {
        namespaces.push(NamespaceStatusModel {
            name: namespace.name.to_string(),
            connected_to_main_node: namespace.main_node.is_connected(),
            main_node_ping: namespace.main_node.get_ping_micros(),
        });

        add_tables(&mut tables, namespace.as_ref());
    }

    let status_bar = build_status_bar(app, &readers, tables.len(), &namespaces);

    StatusModel {
        // The node serves from the start - it is never in the initialization state the main
        // node reports while it loads its tables.
        not_initialized: false,
        initialized: InitializedModel {
            readers: readers.models,
            tables,
            namespaces,
        },
        status_bar,
    }
}

/// Every table the readers of the namespace subscribed to. One asked for but not arrived yet is
/// listed too - with no data - since a reader waiting for it is exactly what is worth seeing.
fn add_tables(tables: &mut Vec<TableModel>, namespace: &NodeNamespace) {
    for (table_name, state) in namespace.main_node.get_requested_tables() {
        let sync_state = match state {
            TableRequestState::Pending => TABLE_SYNC_PENDING,
            TableRequestState::Replicated => TABLE_SYNC_REPLICATED,
            TableRequestState::NotFound => TABLE_SYNC_NOT_FOUND,
        };

        let metrics = namespace
            .db
            .get_table(table_name.as_str())
            .map(|db_table| crate::operations::get_table_metrics(db_table.as_ref()));

        tables.push(TableModel {
            namespace: namespace.name.to_string(),
            name: table_name,
            sync_state: sync_state.to_string(),
            partitions_count: metrics.as_ref().map(|m| m.partitions_amount).unwrap_or(0) as u64,
            data_size: metrics.as_ref().map(|m| m.table_size).unwrap_or(0) as u64,
            records_amount: metrics.as_ref().map(|m| m.records_amount).unwrap_or(0) as u64,
        });
    }
}

fn build_status_bar(
    app: &AppContext,
    readers: &Readers,
    tables_amount: usize,
    namespaces: &[NamespaceStatusModel],
) -> StatusBarModel {
    StatusBarModel {
        location: LocationModel {
            id: app.settings.location.to_string(),
            compress: app.settings.compress,
        },
        version: crate::app::APP_VERSION.to_string(),
        tcp_connections: readers.tcp_count,
        http_connections: readers.http_count,
        tables_amount: tables_amount as u64,
        // Every namespace the node replicates is connected to the main node.
        connected_to_main_node: namespaces
            .iter()
            .all(|namespace| namespace.connected_to_main_node),
        // The slowest ping to the main node among the namespaces, microseconds.
        main_node_ping: namespaces
            .iter()
            .map(|namespace| namespace.main_node_ping)
            .max()
            .unwrap_or(0),
    }
}

struct Readers {
    models: Vec<ReaderModel>,
    tcp_count: u64,
    http_count: u64,
}

fn get_readers(app: &AppContext) -> Readers {
    let now = DateTimeAsMicroseconds::now();

    let mut result = Readers {
        models: Vec::new(),
        tcp_count: 0,
        http_count: 0,
    };

    for data_reader in app.data_readers.get_all() {
        match &data_reader.connection {
            DataReaderConnection::Tcp(_) => result.tcp_count += 1,
            DataReaderConnection::Http(_) => result.http_count += 1,
        }

        let metrics = data_reader.get_metrics();

        // A connection which has not introduced itself yet is not a reader so far.
        let Some(name) = metrics.name else {
            continue;
        };

        result.models.push(ReaderModel {
            id: metrics.session_id,
            name,
            namespace: metrics.namespace,
            ip: metrics.ip,
            tables: metrics.tables,
            awaiting_tables: metrics.awaiting_tables,
            last_incoming_time: format!(
                "{:?}",
                now.duration_since(metrics.last_incoming_moment)
                    .as_positive_or_zero()
            ),
            connected_time: metrics.connected.to_rfc3339(),
            pending_to_send: metrics.pending_to_send as u64,
            sent_per_second: data_reader
                .get_sent_per_second()
                .into_iter()
                .map(|itm| itm as u64)
                .collect(),
            latency: metrics.latency,
        });
    }

    result
}
