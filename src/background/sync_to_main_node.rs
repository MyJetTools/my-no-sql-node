use crate::{app::AppContext, tcp_client_to_main_node::DataReaderTcpConnection};
use my_no_sql_core::sync_to_main::{DeliverToMainNodeEvent, SyncToMainNodeEvent};
use rust_extensions::events_loop::EventsLoopTick;
use std::sync::Arc;

pub struct SyncToMainNodeEventLoop {
    app: Arc<AppContext>,
}

impl SyncToMainNodeEventLoop {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl EventsLoopTick<SyncToMainNodeEvent<DataReaderTcpConnection>> for SyncToMainNodeEventLoop {
    async fn tick(&self, event: SyncToMainNodeEvent<DataReaderTcpConnection>) {
        match event {
            SyncToMainNodeEvent::Connected(connection) => {
                self.app
                    .sync_to_main_node_queue
                    .new_connection(connection)
                    .await;

                to_main_node_pusher(&self.app, None).await;
            }
            SyncToMainNodeEvent::Disconnected(_) => {
                self.app.sync_to_main_node_queue.disconnected().await;
            }
            SyncToMainNodeEvent::PingToDeliver => {
                to_main_node_pusher(&self.app, None).await;
            }
            SyncToMainNodeEvent::Delivered(confirmation_id) => {
                to_main_node_pusher(&self.app, Some(confirmation_id)).await;
            }
        }
    }
}

pub async fn to_main_node_pusher(app: &Arc<AppContext>, delivered_confimration_id: Option<i64>) {
    use my_no_sql_tcp_shared::MyNoSqlTcpContract;
    let next_event = app
        .sync_to_main_node_queue
        .get_next_event_to_deliver(delivered_confimration_id)
        .await;

    if next_event.is_none() {
        return;
    }

    let (connection, next_event) = next_event.unwrap();

    match next_event {
        DeliverToMainNodeEvent::UpdatePartitionsExpiration {
            event,
            confirmation_id,
        } => {
            let mut partitions = Vec::with_capacity(event.partitions.len());

            for (partition, expiration_time) in event.partitions {
                partitions.push((partition, expiration_time));
            }

            connection
                .send(MyNoSqlTcpContract::UpdatePartitionsExpirationTime {
                    confirmation_id,
                    table_name: event.table_name,
                    partitions,
                })
                .await;
        }
        DeliverToMainNodeEvent::UpdatePartitionsLastReadTime {
            event,
            confirmation_id,
        } => {
            let mut partitions = Vec::with_capacity(event.partitions.len());

            for (partition, _) in event.partitions {
                partitions.push(partition);
            }

            connection
                .send(MyNoSqlTcpContract::UpdatePartitionsLastReadTime {
                    confirmation_id,
                    table_name: event.table_name,
                    partitions,
                })
                .await;
        }
        DeliverToMainNodeEvent::UpdateRowsExpirationTime {
            event,
            confirmation_id,
        } => {
            let mut row_keys = Vec::with_capacity(event.row_keys.len());

            for (row_key, _) in event.row_keys {
                row_keys.push(row_key);
            }

            connection
                .send(MyNoSqlTcpContract::UpdateRowsExpirationTime {
                    confirmation_id,
                    table_name: event.table_name,
                    partition_key: event.partition_key,
                    row_keys,
                    expiration_time: event.expiration_time,
                })
                .await;
        }
        DeliverToMainNodeEvent::UpdateRowsLastReadTime {
            event,
            confirmation_id,
        } => {
            let mut row_keys = Vec::with_capacity(event.row_keys.len());

            for (row_key, _) in event.row_keys {
                row_keys.push(row_key);
            }

            connection
                .send(MyNoSqlTcpContract::UpdateRowsLastReadTime {
                    confirmation_id,
                    table_name: event.table_name,
                    partition_key: event.partition_key,
                    row_keys,
                })
                .await;
        }
    }
}
