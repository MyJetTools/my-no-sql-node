use std::sync::Arc;

use my_logger::LogEventCtx;
use my_no_sql_sdk::tcp_contracts::{
    sync_to_main::UpdateEntityStatisticsData, MyNoSqlReaderTcpSerializer, MyNoSqlTcpContract,
};
use my_tcp_sockets::{tcp_connection::TcpSocketConnection, SocketEventCallback};

use crate::app::AppContext;

pub type MyNoSqlTcpConnection =
    TcpSocketConnection<MyNoSqlTcpContract, MyNoSqlReaderTcpSerializer, ()>;

#[derive(Clone)]
pub struct TcpServerEvents {
    app: Arc<AppContext>,
}

impl TcpServerEvents {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl SocketEventCallback<MyNoSqlTcpContract, MyNoSqlReaderTcpSerializer, ()> for TcpServerEvents {
    async fn connected(
        &mut self,
        connection: Arc<TcpSocketConnection<MyNoSqlTcpContract, MyNoSqlReaderTcpSerializer, ()>>,
    ) {
        my_logger::LOGGER.write_info(
            "ServerConnection::connected",
            "New connection established",
            LogEventCtx::new().add("ConnectionId", connection.id.to_string()),
        );

        self.app.data_readers.add_tcp(connection).await;
        self.app.metrics.mark_new_tcp_connection();
    }
    async fn disconnected(
        &mut self,
        connection: Arc<TcpSocketConnection<MyNoSqlTcpContract, MyNoSqlReaderTcpSerializer, ()>>,
    ) {
        my_logger::LOGGER.write_info(
            "ServerConnection::disconnected",
            "Connection lost",
            LogEventCtx::new().add("ConnectionId", connection.id.to_string()),
        );
        if let Some(data_reader) = self.app.data_readers.remove_tcp(connection.as_ref()).await {
            self.app
                .metrics
                .remove_pending_to_sync(&data_reader.connection)
                .await;
        }
        self.app.metrics.mark_new_tcp_disconnection();
    }

    async fn payload(
        &mut self,
        connection: &Arc<TcpSocketConnection<MyNoSqlTcpContract, MyNoSqlReaderTcpSerializer, ()>>,
        contract: MyNoSqlTcpContract,
    ) {
        match contract {
            MyNoSqlTcpContract::Ping => {
                connection.send(&MyNoSqlTcpContract::Pong);
            }
            MyNoSqlTcpContract::Greeting { name } => {
                if let Some(data_reader) = self.app.data_readers.get_tcp(connection.as_ref()).await
                {
                    my_logger::LOGGER.write_info(
                        "ServerConnection::payload",
                        "Connection name updated",
                        LogEventCtx::new()
                            .add("ConnectionId", connection.id.to_string())
                            .add("ConnectionName", name.to_string()),
                    );
                    data_reader.set_name_as_reader(name).await;
                }
            }

            MyNoSqlTcpContract::Subscribe { table_name } => {
                if let Some(data_reader) = self.app.data_readers.get_tcp(connection.as_ref()).await
                {
                    let result = crate::operations::data_readers::subscribe(
                        self.app.as_ref(),
                        data_reader,
                        &table_name,
                    )
                    .await;

                    if let Err(err) = result {
                        let session = self.app.data_readers.get_tcp(connection.as_ref()).await;

                        let session_name = if let Some(session) = session {
                            session.get_name().await
                        } else {
                            None
                        };

                        let message = format!(
                            "Session: {:?}. Subscribe to table {} error. Err: {:?}",
                            session_name, table_name, err
                        );

                        my_logger::LOGGER.write_info(
                            "ServerConnection::payload",
                            message.clone(),
                            LogEventCtx::new()
                                .add("ConnectionId", connection.id.to_string())
                                .add("TableName", table_name.to_string()),
                        );

                        connection.send(&MyNoSqlTcpContract::Error { message });
                    }
                }
            }
            MyNoSqlTcpContract::UpdatePartitionsExpirationTime {
                confirmation_id,
                table_name,
                partitions,
            } => {
                for (partition_key, expiration_time) in partitions {
                    self.app.sync_to_main_node.update(
                        table_name.as_str(),
                        &partition_key,
                        || [].into_iter(),
                        &UpdateEntityStatisticsData {
                            partition_last_read_moment: false,
                            row_last_read_moment: false,
                            partition_expiration_moment: Some(expiration_time),
                            row_expiration_moment: None,
                        },
                    );
                }

                connection.send(&MyNoSqlTcpContract::Confirmation { confirmation_id });
            }

            MyNoSqlTcpContract::UpdateRowsExpirationTime {
                confirmation_id,
                table_name,
                partition_key,
                row_keys,
                expiration_time,
            } => {
                self.app.sync_to_main_node.update(
                    table_name.as_str(),
                    &partition_key,
                    || row_keys.iter().map(|itm| itm.as_str()),
                    &UpdateEntityStatisticsData {
                        partition_last_read_moment: false,
                        row_last_read_moment: false,
                        partition_expiration_moment: None,
                        row_expiration_moment: Some(expiration_time),
                    },
                );

                connection.send(&MyNoSqlTcpContract::Confirmation { confirmation_id });
            }

            MyNoSqlTcpContract::UpdateRowsLastReadTime {
                confirmation_id,
                table_name,
                partition_key,
                row_keys,
            } => {
                self.app.sync_to_main_node.update(
                    table_name.as_str(),
                    &partition_key,
                    || row_keys.iter().map(|itm| itm.as_str()),
                    &UpdateEntityStatisticsData {
                        partition_last_read_moment: false,
                        row_last_read_moment: true,
                        partition_expiration_moment: None,
                        row_expiration_moment: None,
                    },
                );

                connection.send(&MyNoSqlTcpContract::Confirmation { confirmation_id });
            }
            MyNoSqlTcpContract::UpdatePartitionsLastReadTime {
                confirmation_id,
                table_name,
                partitions,
            } => {
                for partition_key in partitions {
                    self.app.sync_to_main_node.update(
                        table_name.as_str(),
                        &partition_key,
                        || [].into_iter(),
                        &UpdateEntityStatisticsData {
                            partition_last_read_moment: true,
                            row_last_read_moment: false,
                            partition_expiration_moment: None,
                            row_expiration_moment: None,
                        },
                    );
                }

                connection.send(&MyNoSqlTcpContract::Confirmation { confirmation_id });
            }
            _ => {}
        }
    }
}
