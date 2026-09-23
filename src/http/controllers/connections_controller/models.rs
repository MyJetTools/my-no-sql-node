use my_no_sql_sdk::core::rust_extensions::date_time::DateTimeAsMicroseconds;
use rest_api_shared::{ConnectionReaderContract, ConnectionsContract, MainNodeConnectionContract};

use crate::app::AppContext;

pub fn build_connections(app: &AppContext) -> ConnectionsContract {
    let now = DateTimeAsMicroseconds::now();

    let mut outgoing_per_second = 0;

    let mut readers = Vec::new();

    for data_reader in app.data_readers.get_all() {
        let metrics = data_reader.get_metrics();

        // A connection which has not introduced itself yet is not a reader so far.
        let Some(name) = metrics.name else {
            continue;
        };

        let outgoing = data_reader.get_outgoing_per_second();
        outgoing_per_second += outgoing.unwrap_or(0);

        readers.push(ConnectionReaderContract {
            id: metrics.session_id,
            name,
            namespace: metrics.namespace,
            ip: metrics.ip,
            tables: metrics.tables,
            awaiting_tables: metrics.awaiting_tables,
            outgoing_per_second: outgoing.map(|itm| itm as u64),
            pending_to_send: metrics.pending_to_send as u64,
            last_incoming_time: format!(
                "{:?}",
                now.duration_since(metrics.last_incoming_moment)
                    .as_positive_or_zero()
            ),
        });
    }

    let main_nodes = app
        .namespaces
        .get_all()
        .iter()
        .map(|namespace| {
            let traffic = namespace.main_node.get_traffic_per_second();

            MainNodeConnectionContract {
                namespace: namespace.name.to_string(),
                connected: namespace.main_node.is_connected(),
                ping: namespace.main_node.get_ping_micros(),
                incoming_per_second: traffic.incoming as u64,
                outgoing_per_second: traffic.outgoing as u64,
            }
        })
        .collect();

    ConnectionsContract {
        outgoing_per_second: outgoing_per_second as u64,
        readers,
        main_nodes,
    }
}
