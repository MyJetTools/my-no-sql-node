use std::sync::{atomic::Ordering, Arc};

use my_no_sql_tcp_shared::{MyNoSqlReaderTcpSerializer, TcpContract};
use my_tcp_sockets::{ConnectionEvent, SocketEventCallback};

use crate::{app::AppContext, db_sync::EventSource};

pub type TcpConnection =
    my_tcp_sockets::tcp_connection::SocketConnection<TcpContract, MyNoSqlReaderTcpSerializer>;

pub struct TcpClientSocketCallback {
    app: Arc<AppContext>,
}

impl TcpClientSocketCallback {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }

    pub async fn handle_incoming_packet(
        &self,
        tcp_contract: TcpContract,
        connection: Arc<TcpConnection>,
    ) {
        match tcp_contract {
            TcpContract::Pong => {
                if let Some(duration) = connection.statistics.get_ping_pong_duration() {
                    let microseconds = duration.as_micros();
                    self.app
                        .master_node_ping_interval
                        .store(microseconds as i64, Ordering::SeqCst);
                }
            }
            TcpContract::InitTable { table_name, data } => {
                crate::db_operations::sync_from_main::sync_table(
                    &self.app,
                    table_name,
                    data,
                    EventSource::SyncFromMain,
                )
                .await;
            }
            TcpContract::InitPartition {
                table_name,
                partition_key,
                data,
            } => {
                crate::db_operations::sync_from_main::sync_partition(
                    &self.app,
                    table_name,
                    partition_key,
                    data,
                    EventSource::SyncFromMain,
                )
                .await;
            }
            TcpContract::UpdateRows { table_name, data } => {
                crate::db_operations::sync_from_main::sync_rows(
                    &self.app,
                    table_name,
                    data,
                    EventSource::SyncFromMain,
                )
                .await;
            }
            TcpContract::DeleteRows { table_name, rows } => {
                crate::db_operations::sync_from_main::delete_rows(
                    &self.app,
                    table_name,
                    rows,
                    EventSource::SyncFromMain,
                )
                .await;
            }
            TcpContract::Error { message } => {
                self.app.logs.add_error(
                    None,
                    crate::app::logs::SystemProcess::TcpSocket,
                    "TcoClientError".to_string(),
                    message,
                    Some(format!("{:?}", connection.id)),
                );
            }
            _ => {}
        }
    }
}

#[async_trait::async_trait]
impl SocketEventCallback<TcpContract, MyNoSqlReaderTcpSerializer> for TcpClientSocketCallback {
    async fn handle(
        &self,
        connection_event: ConnectionEvent<TcpContract, MyNoSqlReaderTcpSerializer>,
    ) {
        match connection_event {
            ConnectionEvent::Connected(connection) => {
                let contract = TcpContract::GreetingFromNode {
                    node_location: self.app.settings.location.to_string(),
                    node_version: crate::app::APP_VERSION.to_string(),
                };

                connection.send(contract).await;

                for table in &self.app.settings.tables {
                    let contract = TcpContract::Subscribe {
                        table_name: table.to_string(),
                    };

                    connection.send(contract).await;
                }
                self.app
                    .connected_to_main_node
                    .store(true, Ordering::SeqCst);
            }
            ConnectionEvent::Disconnected(_connection) => {
                self.app
                    .connected_to_main_node
                    .store(false, Ordering::SeqCst);
            }
            ConnectionEvent::Payload {
                connection,
                payload,
            } => self.handle_incoming_packet(payload, connection).await,
        }
    }
}
