use std::sync::Arc;

use my_no_sql_sdk::{
    core::rust_extensions::events_loop::{EventsLoopTick, RepeatIteration},
    tcp_contracts::MyNoSqlTcpContract,
};

use crate::{
    app::AppContext,
    data_readers::DataReaderConnection,
    db_sync::{NamespaceSyncEvent, SyncEvent},
};

/// Delivers the changes to the readers.
pub struct SyncEventsToClients {
    app: Arc<AppContext>,
}

impl SyncEventsToClients {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }

    fn sync_to_clients(&self, model: &NamespaceSyncEvent) {
        let sync_event = &model.event;

        if let SyncEvent::TableFirstInit(data) = sync_event {
            data.data_reader.set_first_init();

            match &data.data_reader.connection {
                DataReaderConnection::Tcp(tcp_info) => {
                    tcp_info.send(crate::data_readers::compile_tcp_payload(sync_event).as_slice());
                }
                DataReaderConnection::Http(http_info) => {
                    http_info.send(sync_event);
                }
            }

            self.app
                .metrics
                .update_pending_to_sync(&data.data_reader.connection);

            return;
        }

        let data_readers = self
            .app
            .data_readers
            .get_subscribed_to_table(&model.namespace, sync_event.get_table_name());

        // Serialized once and only if some TCP reader needs it.
        let mut tcp_contracts: Option<Vec<MyNoSqlTcpContract>> = None;

        for data_reader in data_readers.iter() {
            // It has not got the table yet: the snapshot it is going to get is taken later and
            // carries this change already.
            if !data_reader.has_first_init() {
                continue;
            }

            match &data_reader.connection {
                DataReaderConnection::Tcp(tcp_info) => {
                    let tcp_contracts = tcp_contracts.get_or_insert_with(|| {
                        crate::data_readers::compile_tcp_payload(sync_event)
                    });

                    tcp_info.send(tcp_contracts.as_slice());
                }
                DataReaderConnection::Http(http_info) => {
                    http_info.send(sync_event);
                }
            }

            self.app
                .metrics
                .update_pending_to_sync(&data_reader.connection);
        }
    }
}

#[async_trait::async_trait]
impl EventsLoopTick<NamespaceSyncEvent> for SyncEventsToClients {
    async fn started(&self) {}

    async fn tick(&self, model: NamespaceSyncEvent) -> RepeatIteration<NamespaceSyncEvent> {
        self.sync_to_clients(&model);
        RepeatIteration::No
    }

    async fn finished(&self) {}
}
