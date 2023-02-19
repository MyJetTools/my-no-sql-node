use std::{
    collections::HashMap,
    sync::{atomic::Ordering, Arc},
};

use my_no_sql_tcp_shared::{MyNoSqlReaderTcpSerializer, MyNoSqlTcpContract};
use my_tcp_sockets::{ConnectionEvent, SocketEventCallback};

use crate::{app::AppContext, db_sync::EventSource};

pub type DataReaderTcpConnection = my_tcp_sockets::tcp_connection::SocketConnection<
    MyNoSqlTcpContract,
    MyNoSqlReaderTcpSerializer,
>;

pub struct TcpClientSocketCallback {
    app: Arc<AppContext>,
}

impl TcpClientSocketCallback {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }

    pub async fn handle_incoming_packet(
        &self,
        tcp_contract: MyNoSqlTcpContract,
        connection: Arc<DataReaderTcpConnection>,
    ) {
        match tcp_contract {
            MyNoSqlTcpContract::Pong => {
                if let Some(duration) = connection.statistics.get_ping_pong_duration() {
                    let microseconds = duration.as_micros();
                    self.app
                        .master_node_ping_interval
                        .store(microseconds as i64, Ordering::SeqCst);
                }
            }
            MyNoSqlTcpContract::InitTable { table_name, data } => {
                crate::db_operations::sync_from_main::sync_table(
                    &self.app,
                    table_name,
                    data,
                    EventSource::SyncFromMain,
                )
                .await;
            }
            MyNoSqlTcpContract::InitPartition {
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
            MyNoSqlTcpContract::UpdateRows { table_name, data } => {
                crate::db_operations::sync_from_main::sync_rows(
                    &self.app,
                    table_name,
                    data,
                    EventSource::SyncFromMain,
                )
                .await;
            }
            MyNoSqlTcpContract::DeleteRows { table_name, rows } => {
                crate::db_operations::sync_from_main::delete_rows(
                    &self.app,
                    table_name,
                    rows,
                    EventSource::SyncFromMain,
                )
                .await;
            }
            MyNoSqlTcpContract::Error { message } => {
                let mut ctx = HashMap::new();
                ctx.insert("connection_id".to_string(), connection.id.to_string());
                self.app.logs.add_error(
                    None,
                    my_no_sql_server_core::logs::SystemProcess::TcpSocket,
                    "TcoClientError".to_string(),
                    message,
                    Some(ctx),
                );
            }
            MyNoSqlTcpContract::TableNotFound(table_name) => {
                let data_readers = self.app.data_readers.get_all().await;

                for data_reader in data_readers {
                    if data_reader.has_awaiting_table(table_name.as_str()).await {
                        data_reader
                            .send_error_to_client(format!("Table {} not found", table_name))
                            .await
                    }
                }
            }

            MyNoSqlTcpContract::Confirmation { confirmation_id } => {
                self.app.sync_to_main_node_events_loop.send(
                    crate::background::sync_to_main_node::SyncToMainNodeEvent::Delivered(
                        confirmation_id,
                    ),
                );
            }
            _ => {}
        }
    }
}

#[async_trait::async_trait]
impl SocketEventCallback<MyNoSqlTcpContract, MyNoSqlReaderTcpSerializer>
    for TcpClientSocketCallback
{
    async fn handle(
        &self,
        connection_event: ConnectionEvent<MyNoSqlTcpContract, MyNoSqlReaderTcpSerializer>,
    ) {
        match connection_event {
            ConnectionEvent::Connected(connection) => {
                let contract = MyNoSqlTcpContract::GreetingFromNode {
                    node_location: self.app.settings.location.to_string(),
                    node_version: crate::app::APP_VERSION.to_string(),
                    compress: self.app.settings.compress,
                };

                connection.send(contract).await;

                let tables = self.app.db.get_tables().await;

                for table in tables {
                    let contract = MyNoSqlTcpContract::SubscribeAsNode(table.name.to_string());
                    connection.send(contract).await;
                }

                self.app
                    .connected_to_main_node
                    .connected(connection.clone())
                    .await;

                self.app.sync_to_main_node_events_loop.send(
                    crate::background::sync_to_main_node::SyncToMainNodeEvent::Connected(
                        connection,
                    ),
                );
            }
            ConnectionEvent::Disconnected(connection) => {
                self.app.connected_to_main_node.disconnected().await;

                self.app.sync_to_main_node_events_loop.send(
                    crate::background::sync_to_main_node::SyncToMainNodeEvent::Disconnected(
                        connection,
                    ),
                );
            }
            ConnectionEvent::Payload {
                connection,
                payload,
            } => {
                if let MyNoSqlTcpContract::CompressedPayload(data) = &payload {
                    println!("CompressedPayload: {}", data.len());
                }
                let payload = payload.decompress_if_compressed().await.unwrap();

                self.handle_incoming_packet(payload, connection).await;
            }
        }
    }
}
