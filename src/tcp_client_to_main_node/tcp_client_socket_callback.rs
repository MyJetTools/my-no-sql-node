use std::{
    collections::HashMap,
    sync::{atomic::Ordering, Arc},
};

use my_logger::LogEventCtx;
use my_no_sql_sdk::tcp_contracts::{MyNoSqlReaderTcpSerializer, MyNoSqlTcpContract};
use my_tcp_sockets::{tcp_connection::TcpSocketConnection, SocketEventCallback};

use crate::{app::AppContext, tcp_server::MyNoSqlTcpConnection};

#[derive(Clone)]
pub struct TcpClientSocketCallback {
    app: Arc<AppContext>,
}

impl TcpClientSocketCallback {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl SocketEventCallback<MyNoSqlTcpContract, MyNoSqlReaderTcpSerializer, ()>
    for TcpClientSocketCallback
{
    async fn connected(
        &mut self,
        connection: Arc<TcpSocketConnection<MyNoSqlTcpContract, MyNoSqlReaderTcpSerializer, ()>>,
    ) {
        let contract = MyNoSqlTcpContract::GreetingFromNode {
            node_location: self.app.settings.location.to_string(),
            node_version: crate::app::APP_VERSION.to_string(),
            compress: self.app.settings.compress,
        };

        connection.send(&contract);

        let tables = self.app.db.get_tables();

        for table in tables.iter() {
            let contract = MyNoSqlTcpContract::SubscribeAsNode(table.name.to_string());
            connection.send(&contract);
        }

        self.app
            .connected_to_main_node
            .connected(connection.clone())
            .await;

        self.app
            .sync_to_main_node
            .tcp_events_pusher_new_connection_established(connection);
    }

    async fn disconnected(&mut self, connection: Arc<MyNoSqlTcpConnection>) {
        self.app.connected_to_main_node.disconnected().await;

        self.app
            .sync_to_main_node
            .tcp_events_pusher_connection_disconnected(connection);
    }

    async fn payload(
        &mut self,
        connection: &Arc<MyNoSqlTcpConnection>,
        contract: MyNoSqlTcpContract,
    ) {
        if let MyNoSqlTcpContract::CompressedPayload(data) = &contract {
            println!("CompressedPayload: {}", data.len());
        }
        let contract = contract.decompress_if_compressed().await.unwrap();

        match contract {
            MyNoSqlTcpContract::Pong => {
                if let Some(duration) = connection.statistics().get_ping_pong_duration() {
                    let microseconds = duration.as_micros();
                    self.app
                        .master_node_ping_interval
                        .store(microseconds as i64, Ordering::SeqCst);
                }
            }
            MyNoSqlTcpContract::InitTable { table_name, data } => {
                crate::db_operations::sync_from_main::sync_table(&self.app, table_name, data).await;
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
                )
                .await;
            }
            MyNoSqlTcpContract::UpdateRows { table_name, data } => {
                crate::db_operations::sync_from_main::sync_rows(&self.app, table_name, data).await;
            }
            MyNoSqlTcpContract::DeleteRows { table_name, rows } => {
                crate::db_operations::sync_from_main::delete_rows(&self.app, table_name, rows)
                    .await;
            }
            MyNoSqlTcpContract::Error { message } => {
                let mut ctx = HashMap::new();
                ctx.insert("connection_id".to_string(), connection.id.to_string());

                my_logger::LOGGER.write_error(
                    "TcpPayload",
                    message,
                    LogEventCtx::new().add("ConnectionId", connection.id.to_string()),
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
                self.app
                    .sync_to_main_node
                    .tcp_events_pusher_got_confirmation(confirmation_id);
            }

            _ => {}
        }
    }
}
