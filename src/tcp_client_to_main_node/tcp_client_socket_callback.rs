use std::sync::Arc;

use my_logger::LogEventCtx;
use my_no_sql_sdk::tcp_contracts::{MyNoSqlReaderTcpSerializer, MyNoSqlTcpContract};
use my_tcp_sockets::SocketEventCallback;

use crate::{app::AppContext, namespaces::NodeNamespace, tcp_server::MyNoSqlTcpConnection};

/// Events of the connection one namespace keeps to the main node.
pub struct TcpClientSocketCallback {
    app: Arc<AppContext>,
    namespace: Arc<NodeNamespace>,
}

impl TcpClientSocketCallback {
    pub fn new(app: Arc<AppContext>, namespace: Arc<NodeNamespace>) -> Self {
        Self { app, namespace }
    }
}

#[async_trait::async_trait]
impl SocketEventCallback<MyNoSqlTcpContract, MyNoSqlReaderTcpSerializer, ()>
    for TcpClientSocketCallback
{
    async fn connected(&mut self, connection: Arc<MyNoSqlTcpConnection>) {
        connection.send(&MyNoSqlTcpContract::GreetingFromNode {
            node_location: self.app.settings.location.to_string(),
            node_version: crate::app::APP_VERSION.to_string(),
            compress: self.app.settings.compress,
        });

        // Right after the Greeting and before the first subscription. The default namespace is
        // not sent: it is what the main node uses anyway, and staying silent about it keeps a
        // main node which knows nothing about namespaces working.
        if !self.namespace.name.is_default() {
            connection.send(&MyNoSqlTcpContract::SetNamespace {
                namespace: self.namespace.name.to_string(),
            });
        }

        self.namespace.main_node.connected(&connection);

        self.namespace
            .main_node
            .sync_to_main_node
            .tcp_events_pusher_new_connection_established(connection);
    }

    async fn disconnected(&mut self, connection: Arc<MyNoSqlTcpConnection>) {
        self.namespace.main_node.disconnected();

        self.namespace
            .main_node
            .sync_to_main_node
            .tcp_events_pusher_connection_disconnected(connection);
    }

    async fn payload(
        &mut self,
        connection: &Arc<MyNoSqlTcpConnection>,
        contract: MyNoSqlTcpContract,
    ) {
        match contract {
            MyNoSqlTcpContract::Pong => {
                if let Some(ping_duration) = connection.statistics().get_ping_pong_duration() {
                    self.namespace.main_node.update_ping(ping_duration);
                }
            }
            MyNoSqlTcpContract::InitTable { table_name, data } => {
                self.namespace.main_node.add_received_payload(data.len());
                crate::db_operations::sync_from_main::sync_table(
                    &self.app,
                    &self.namespace,
                    table_name,
                    data,
                );
            }
            MyNoSqlTcpContract::InitPartition {
                table_name,
                partition_key,
                data,
            } => {
                self.namespace.main_node.add_received_payload(data.len());
                crate::db_operations::sync_from_main::sync_partition(
                    &self.app,
                    &self.namespace,
                    table_name,
                    partition_key,
                    data,
                );
            }
            MyNoSqlTcpContract::UpdateRows { table_name, data } => {
                self.namespace.main_node.add_received_payload(data.len());
                crate::db_operations::sync_from_main::sync_rows(
                    &self.app,
                    &self.namespace,
                    table_name,
                    data,
                );
            }
            MyNoSqlTcpContract::DeleteRows { table_name, rows } => {
                self.namespace.main_node.add_received_payload(
                    rows.iter()
                        .map(|row| row.partition_key.len() + row.row_key.len())
                        .sum(),
                );
                crate::db_operations::sync_from_main::delete_rows(
                    &self.app,
                    &self.namespace,
                    table_name,
                    rows,
                );
            }
            MyNoSqlTcpContract::TableNotFound(table_name) => {
                crate::db_operations::sync_from_main::table_not_found(
                    &self.app,
                    &self.namespace,
                    table_name,
                );
            }
            MyNoSqlTcpContract::Confirmation { confirmation_id } => {
                self.namespace
                    .main_node
                    .sync_to_main_node
                    .tcp_events_pusher_got_confirmation(confirmation_id);
            }
            MyNoSqlTcpContract::Error { message } => {
                my_logger::LOGGER.write_error(
                    "MainNodeConnection",
                    message,
                    LogEventCtx::new()
                        .add("namespace", self.namespace.name.to_string())
                        .add("connectionId", connection.id.to_string()),
                );
            }
            // The main node never sends these to a node. A CompressedPayload never reaches here
            // either - the serializer inflates it into the packet it carries.
            MyNoSqlTcpContract::Ping
            | MyNoSqlTcpContract::PingWithLatency { .. }
            | MyNoSqlTcpContract::Greeting { .. }
            | MyNoSqlTcpContract::Subscribe { .. }
            | MyNoSqlTcpContract::GreetingFromNode { .. }
            | MyNoSqlTcpContract::SubscribeAsNode(_)
            | MyNoSqlTcpContract::Unsubscribe(_)
            | MyNoSqlTcpContract::CompressedPayload(_)
            | MyNoSqlTcpContract::UpdatePartitionsLastReadTime { .. }
            | MyNoSqlTcpContract::UpdateRowsLastReadTime { .. }
            | MyNoSqlTcpContract::UpdatePartitionsExpirationTime { .. }
            | MyNoSqlTcpContract::UpdateRowsExpirationTime { .. }
            | MyNoSqlTcpContract::SetNamespace { .. } => {}
        }
    }
}
