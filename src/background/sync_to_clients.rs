use std::sync::Arc;

use my_no_sql_sdk::{
    core::rust_extensions::events_loop::EventsLoopTick, tcp_contracts::MyNoSqlTcpContract,
};

use crate::{app::AppContext, data_readers::DataReaderConnection, db_sync::SyncEvent};

pub struct SyncEventsToClients {
    app: Arc<AppContext>,
}

impl SyncEventsToClients {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl EventsLoopTick<SyncEvent> for SyncEventsToClients {
    async fn started(&self) {
        println!("Sync to clients event loop started");
    }
    async fn tick(&self, sync_event: SyncEvent) {
        if let SyncEvent::TableFirstInit(data) = &sync_event {
            data.data_reader.set_first_init();

            match &data.data_reader.connection {
                DataReaderConnection::Tcp(tcp_info) => {
                    let payloads_to_send =
                        crate::data_readers::tcp_connection::tcp_payload_to_send::serialize(
                            &sync_event,
                        )
                        .await;

                    for payload_to_send in payloads_to_send {
                        tcp_info.send(&payload_to_send).await;
                    }
                }
                DataReaderConnection::Http(http_info) => {
                    http_info.send(&sync_event).await;
                }
            }

            self.app
                .metrics
                .update_pending_to_sync(&data.data_reader.connection)
                .await;
        } else {
            let data_readers = self
                .app
                .data_readers
                .get_subscribed_to_table(sync_event.get_table_name())
                .await;

            if data_readers.is_none() {
                return;
            }
            let data_readers = data_readers.unwrap();

            let mut tcp_contract_to_send: Option<Vec<MyNoSqlTcpContract>> = None;

            for data_reader in &data_readers {
                if !data_reader.has_first_init() {
                    continue;
                }

                match &data_reader.connection {
                    DataReaderConnection::Tcp(info) => {
                        if tcp_contract_to_send.is_none() {
                            tcp_contract_to_send =
                                crate::data_readers::tcp_connection::tcp_payload_to_send::serialize(
                                    &sync_event,
                                )
                                .await
                                .into();
                        }

                        if let Some(to_send) = &tcp_contract_to_send {
                            for tcp_contract in to_send {
                                info.send(&tcp_contract).await;
                            }
                        }
                    }
                    DataReaderConnection::Http(http_info) => {
                        http_info.send(&sync_event).await;
                    }
                }

                self.app
                    .metrics
                    .update_pending_to_sync(&data_reader.connection)
                    .await;
            }
        }
    }
    async fn finished(&self) {
        println!("Sync to clients event loop finished");
    }
}
