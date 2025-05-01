use std::{
    sync::{atomic::AtomicI64, Arc},
    time::Duration,
};

use my_no_sql_sdk::server::rust_extensions::{events_loop::EventsLoopMutexWrapped, AppStates};
use my_no_sql_sdk::server::DbInstance;
use my_no_sql_sdk::tcp_contracts::sync_to_main::SyncToMainNodeHandler;
use my_tcp_sockets::TcpClient;

use crate::{data_readers::DataReadersList, db_sync::SyncEvent, settings_reader::SettingsModel};

use super::{connection_to_main_node::ConnectionToMainNode, PrometheusMetrics};

pub const APP_VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub struct AppContext {
    pub db: DbInstance,

    pub metrics: PrometheusMetrics,

    pub process_id: String,

    pub data_readers: DataReadersList,

    pub settings: Arc<SettingsModel>,
    pub sync_to_client_events_loop: EventsLoopMutexWrapped<SyncEvent>,
    pub states: Arc<AppStates>,
    pub node_connection_tcp_client: TcpClient,

    pub master_node_ping_interval: AtomicI64,
    pub connected_to_main_node: ConnectionToMainNode,

    pub sync_to_main_node: SyncToMainNodeHandler,
}

impl AppContext {
    pub fn new(settings: Arc<SettingsModel>) -> Self {
        let node_connection_tcp_client =
            TcpClient::new("NodeConnection".to_string(), settings.clone());
        AppContext {
            sync_to_main_node: SyncToMainNodeHandler::new(my_logger::LOGGER.clone()),
            db: DbInstance::new(),
            metrics: PrometheusMetrics::new(),
            process_id: uuid::Uuid::new_v4().to_string(),
            states: Arc::new(AppStates::create_initialized()),

            data_readers: DataReadersList::new(Duration::from_secs(30)),

            settings,
            sync_to_client_events_loop: EventsLoopMutexWrapped::new("SyncEventsLoop".to_string()),
            node_connection_tcp_client,
            master_node_ping_interval: AtomicI64::new(0),
            connected_to_main_node: ConnectionToMainNode::new(),
        }
    }

    pub fn dispatch(&self, sync_event: SyncEvent) {
        self.sync_to_client_events_loop.send(sync_event);
    }
}
