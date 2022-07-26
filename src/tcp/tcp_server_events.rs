use std::sync::Arc;

use my_no_sql_tcp_shared::{MyNoSqlReaderTcpSerializer, TcpContract};
use my_tcp_sockets::{tcp_connection::SocketConnection, ConnectionEvent, SocketEventCallback};

use crate::app::{logs::SystemProcess, AppContext};

pub type MyNoSqlTcpConnection = SocketConnection<TcpContract, MyNoSqlReaderTcpSerializer>;

pub struct TcpServerEvents {
    app: Arc<AppContext>,
}

impl TcpServerEvents {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }

    pub async fn handle_incoming_packet(
        &self,
        tcp_contract: TcpContract,
        connection: Arc<MyNoSqlTcpConnection>,
    ) {
        match tcp_contract {
            TcpContract::Ping => {
                connection.send(TcpContract::Pong).await;
            }
            TcpContract::Greeting { name } => {
                if let Some(data_reader) = self.app.data_readers.get_tcp(connection.as_ref()).await
                {
                    self.app.logs.add_info(
                        None,
                        SystemProcess::TcpSocket,
                        format!("Connection name update to {}", name),
                        format!("ID: {}", connection.id),
                        None,
                    );
                    data_reader.set_name_as_reader(name.to_string()).await;
                }
            }

            TcpContract::GreetingFromNode {
                node_location,
                node_version,
            } => {
                if let Some(data_reader) = self.app.data_readers.get_tcp(connection.as_ref()).await
                {
                    self.app.logs.add_info(
                        None,
                        SystemProcess::TcpSocket,
                        format!(
                            "Connection to node with location {} and version {}",
                            node_location, node_version
                        ),
                        format!("ID: {}", connection.id),
                        None,
                    );
                    data_reader
                        .set_name_as_node(node_location, node_version)
                        .await;
                }
            }

            TcpContract::Subscribe { table_name } => {
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

                        let message =
                            format!("Subscribe to table {} error. Err: {:?}", table_name, err);

                        self.app.logs.add_error(
                            Some(table_name.to_string()),
                            SystemProcess::TcpSocket,
                            "Subscribe to table".to_string(),
                            message.to_string(),
                            Some(format!(
                                "SessionId:{}, Name:{:?}",
                                connection.id, session_name
                            )),
                        );

                        connection.send(TcpContract::Error { message }).await;
                    }
                }
            }
            _ => {}
        }
    }
}

#[async_trait::async_trait]
impl SocketEventCallback<TcpContract, MyNoSqlReaderTcpSerializer> for TcpServerEvents {
    async fn handle(
        &self,
        connection_event: ConnectionEvent<TcpContract, MyNoSqlReaderTcpSerializer>,
    ) {
        match connection_event {
            ConnectionEvent::Connected(connection) => {
                self.app.logs.add_info(
                    None,
                    SystemProcess::TcpSocket,
                    "New tcp connection".to_string(),
                    format!("ID: {}", connection.id),
                    None,
                );

                self.app.data_readers.add_tcp(connection).await;
                self.app.metrics.mark_new_tcp_connection();
            }
            ConnectionEvent::Disconnected(connection) => {
                self.app.logs.add_info(
                    None,
                    SystemProcess::TcpSocket,
                    "Disconnect".to_string(),
                    format!("ID: {}", connection.id),
                    None,
                );
                if let Some(data_reader) =
                    self.app.data_readers.remove_tcp(connection.as_ref()).await
                {
                    self.app
                        .metrics
                        .remove_pending_to_sync(&data_reader.connection)
                        .await;
                }
                self.app.metrics.mark_new_tcp_disconnection();
            }
            ConnectionEvent::Payload {
                connection,
                payload,
            } => self.handle_incoming_packet(payload, connection).await,
        }
    }
}
