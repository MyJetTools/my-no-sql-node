use std::sync::Arc;

use my_logger::LogEventCtx;
use my_no_sql_sdk::tcp_contracts::{
    sync_to_main::UpdateEntityStatisticsData, MyNoSqlReaderTcpSerializer, MyNoSqlTcpContract,
};
use my_tcp_sockets::{tcp_connection::TcpSocketConnection, SocketEventCallback};

use crate::{app::AppContext, data_readers::DataReader, namespaces::NodeNamespace};

pub type MyNoSqlTcpConnection =
    TcpSocketConnection<MyNoSqlTcpContract, MyNoSqlReaderTcpSerializer, ()>;

/// Events of the TCP readers of this node.
#[derive(Clone)]
pub struct TcpServerEvents {
    app: Arc<AppContext>,
}

impl TcpServerEvents {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }

    /// Namespace of the reader, as long as this node replicates the table in it. The statistics
    /// a reader reports belong to a table it is subscribed to - nothing is there to update
    /// otherwise.
    fn get_namespace_of_table(
        &self,
        connection: &MyNoSqlTcpConnection,
        table_name: &str,
    ) -> Option<Arc<NodeNamespace>> {
        let data_reader = self.app.data_readers.get_tcp(connection)?;
        let namespace = self
            .app
            .namespaces
            .get(data_reader.get_namespace().as_str())?;

        namespace.db.get_table(table_name)?;

        Some(namespace)
    }

    fn set_namespace(&self, connection: &MyNoSqlTcpConnection, namespace: String) {
        let Some(data_reader) = self.app.data_readers.get_tcp(connection) else {
            return;
        };

        if let Err(message) = set_namespace(data_reader.as_ref(), namespace.as_str()) {
            my_logger::LOGGER.write_warning(
                "SetNamespace",
                message.as_str(),
                LogEventCtx::new()
                    .add("connectionId", connection.id.to_string())
                    .add("name", format!("{:?}", data_reader.get_name()))
                    .add("namespace", namespace),
            );

            // The answer the main node gives. Going on in another namespace than the one the
            // reader asked for would silently give it the wrong data.
            connection.send(&MyNoSqlTcpContract::Error { message });
        }
    }
}

fn set_namespace(data_reader: &DataReader, namespace: &str) -> Result<(), String> {
    if let Err(err) = my_no_sql_sdk::validate_namespace_name(namespace) {
        return Err(format!("Invalid namespace name. {}", err));
    }

    data_reader.set_namespace(namespace.into())
}

#[async_trait::async_trait]
impl SocketEventCallback<MyNoSqlTcpContract, MyNoSqlReaderTcpSerializer, ()> for TcpServerEvents {
    async fn connected(&mut self, connection: Arc<MyNoSqlTcpConnection>) {
        self.app.data_readers.add_tcp(connection);
        self.app.metrics.mark_new_tcp_connection();
    }

    async fn disconnected(&mut self, connection: Arc<MyNoSqlTcpConnection>) {
        if let Some(data_reader) = self.app.data_readers.remove_tcp(connection.as_ref()) {
            self.app
                .metrics
                .remove_pending_to_sync(&data_reader.connection);
        }

        self.app.metrics.mark_new_tcp_disconnection();
    }

    async fn payload(
        &mut self,
        connection: &Arc<MyNoSqlTcpConnection>,
        contract: MyNoSqlTcpContract,
    ) {
        match contract {
            MyNoSqlTcpContract::Ping => {
                connection.send(&MyNoSqlTcpContract::Pong);
            }
            MyNoSqlTcpContract::Greeting { name } => {
                if let Some(data_reader) = self.app.data_readers.get_tcp(connection.as_ref()) {
                    data_reader.set_name(name);
                }
            }
            MyNoSqlTcpContract::SetNamespace { namespace } => {
                self.set_namespace(connection.as_ref(), namespace);
            }
            MyNoSqlTcpContract::Subscribe { table_name } => {
                let Some(data_reader) = self.app.data_readers.get_tcp(connection.as_ref()) else {
                    return;
                };

                if let Err(err) =
                    crate::operations::subscribe(&self.app, &data_reader, table_name.as_str()).await
                {
                    let message = err.to_string();

                    my_logger::LOGGER.write_warning(
                        "Subscribe",
                        message.as_str(),
                        LogEventCtx::new()
                            .add("connectionId", connection.id.to_string())
                            .add("name", format!("{:?}", data_reader.get_name()))
                            .add("namespace", data_reader.get_namespace().to_string())
                            .add("tableName", table_name),
                    );

                    // Refused loudly: silently not serving the reader would leave it waiting
                    // for data which never comes.
                    connection.send(&MyNoSqlTcpContract::Error { message });
                }
            }
            MyNoSqlTcpContract::Unsubscribe(table_name) => {
                if let Some(data_reader) = self.app.data_readers.get_tcp(connection.as_ref()) {
                    crate::operations::unsubscribe(data_reader.as_ref(), table_name.as_str());
                }
            }
            MyNoSqlTcpContract::UpdatePartitionsLastReadTime {
                confirmation_id,
                table_name,
                partitions,
            } => {
                if let Some(namespace) =
                    self.get_namespace_of_table(connection.as_ref(), table_name.as_str())
                {
                    let update_statistics = UpdateEntityStatisticsData {
                        partition_last_read_moment: true,
                        ..Default::default()
                    };

                    for partition_key in partitions.iter() {
                        namespace.main_node.sync_to_main_node.update(
                            table_name.as_str(),
                            partition_key.as_str(),
                            || [].into_iter(),
                            &update_statistics,
                        );
                    }
                }

                connection.send(&MyNoSqlTcpContract::Confirmation { confirmation_id });
            }
            MyNoSqlTcpContract::UpdateRowsLastReadTime {
                confirmation_id,
                table_name,
                partition_key,
                row_keys,
            } => {
                if let Some(namespace) =
                    self.get_namespace_of_table(connection.as_ref(), table_name.as_str())
                {
                    namespace.main_node.sync_to_main_node.update(
                        table_name.as_str(),
                        partition_key.as_str(),
                        || row_keys.iter().map(|itm| itm.as_str()),
                        &UpdateEntityStatisticsData {
                            row_last_read_moment: true,
                            ..Default::default()
                        },
                    );
                }

                connection.send(&MyNoSqlTcpContract::Confirmation { confirmation_id });
            }
            MyNoSqlTcpContract::UpdatePartitionsExpirationTime {
                confirmation_id,
                table_name,
                partitions,
            } => {
                if let Some(namespace) =
                    self.get_namespace_of_table(connection.as_ref(), table_name.as_str())
                {
                    for (partition_key, expiration_time) in partitions {
                        namespace.main_node.sync_to_main_node.update(
                            table_name.as_str(),
                            partition_key.as_str(),
                            || [].into_iter(),
                            &UpdateEntityStatisticsData {
                                partition_expiration_moment: Some(expiration_time),
                                ..Default::default()
                            },
                        );
                    }
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
                if let Some(namespace) =
                    self.get_namespace_of_table(connection.as_ref(), table_name.as_str())
                {
                    namespace.main_node.sync_to_main_node.update(
                        table_name.as_str(),
                        partition_key.as_str(),
                        || row_keys.iter().map(|itm| itm.as_str()),
                        &UpdateEntityStatisticsData {
                            row_expiration_moment: Some(expiration_time),
                            ..Default::default()
                        },
                    );
                }

                connection.send(&MyNoSqlTcpContract::Confirmation { confirmation_id });
            }
            MyNoSqlTcpContract::GreetingFromNode { node_location, .. } => {
                // Nodes are not chained - a node replicates from the main node only.
                let message = format!(
                    "Node '{}' is connected to a node. Nodes can not be chained - connect it to the main node",
                    node_location
                );

                my_logger::LOGGER.write_warning(
                    "GreetingFromNode",
                    message.as_str(),
                    LogEventCtx::new().add("connectionId", connection.id.to_string()),
                );

                connection.send(&MyNoSqlTcpContract::Error { message });
            }
            // Only a node subscribes this way, and a node is refused on its greeting. The rest
            // are packets a reader never sends.
            MyNoSqlTcpContract::SubscribeAsNode(_)
            | MyNoSqlTcpContract::Pong
            | MyNoSqlTcpContract::InitTable { .. }
            | MyNoSqlTcpContract::InitPartition { .. }
            | MyNoSqlTcpContract::UpdateRows { .. }
            | MyNoSqlTcpContract::DeleteRows { .. }
            | MyNoSqlTcpContract::Error { .. }
            | MyNoSqlTcpContract::TableNotFound(_)
            | MyNoSqlTcpContract::CompressedPayload(_)
            | MyNoSqlTcpContract::Confirmation { .. } => {}
        }
    }
}
