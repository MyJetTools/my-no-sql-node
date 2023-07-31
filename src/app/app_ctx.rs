use std::{
    collections::HashMap,
    sync::{atomic::AtomicI64, Arc},
    time::Duration,
};

use my_no_sql_server_core::{logs::*, DbInstance};
use my_no_sql_tcp_shared::sync_to_main::SyncToMainNodeHandler;
use my_tcp_sockets::TcpClient;
use rust_extensions::{
    date_time::DateTimeAsMicroseconds, events_loop::EventsLoop, AppStates, Logger,
};

use crate::{
    data_readers::DataReadersList, db_operations::multipart::MultipartList, db_sync::SyncEvent,
    settings_reader::SettingsModel,
};

use super::{connection_to_main_node::ConnectionToMainNode, PrometheusMetrics};

pub const APP_VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub struct AppContext {
    pub created: DateTimeAsMicroseconds,
    pub db: DbInstance,
    pub logs: Arc<Logs>,

    pub metrics: PrometheusMetrics,

    pub process_id: String,

    pub data_readers: DataReadersList,

    pub multipart_list: MultipartList,
    pub settings: Arc<SettingsModel>,
    pub sync_to_client_events_loop: EventsLoop<SyncEvent>,
    pub states: Arc<AppStates>,
    pub node_connection_tcp_client: TcpClient,

    pub master_node_ping_interval: AtomicI64,
    pub connected_to_main_node: ConnectionToMainNode,

    pub sync_to_main_node: SyncToMainNodeHandler,
}

impl AppContext {
    pub fn new(logs: Arc<Logs>, settings: Arc<SettingsModel>) -> Self {
        let node_connection_tcp_client =
            TcpClient::new("NodeConnection".to_string(), settings.clone());
        AppContext {
            created: DateTimeAsMicroseconds::now(),
            db: DbInstance::new(),
            logs,
            metrics: PrometheusMetrics::new(),
            process_id: uuid::Uuid::new_v4().to_string(),
            states: Arc::new(AppStates::create_initialized()),

            data_readers: DataReadersList::new(Duration::from_secs(30)),
            multipart_list: MultipartList::new(),

            settings,
            sync_to_client_events_loop: EventsLoop::new("SyncEventsLoop".to_string()),
            node_connection_tcp_client,
            master_node_ping_interval: AtomicI64::new(0),
            connected_to_main_node: ConnectionToMainNode::new(),
            sync_to_main_node: SyncToMainNodeHandler::new(),
        }
    }
}

impl Logger for AppContext {
    fn write_info(
        &self,
        process_name: String,
        message: String,
        context: Option<HashMap<String, String>>,
    ) {
        self.logs
            .add_info(None, SystemProcess::System, process_name, message, context);
    }

    fn write_error(
        &self,
        process_name: String,
        message: String,
        context: Option<HashMap<String, String>>,
    ) {
        self.logs
            .add_fatal_error(None, SystemProcess::System, process_name, message, context);
    }

    fn write_warning(
        &self,
        process_name: String,
        message: String,
        ctx: Option<HashMap<String, String>>,
    ) {
        self.logs
            .add_error(None, SystemProcess::System, process_name, message, ctx);
    }

    fn write_fatal_error(
        &self,
        process_name: String,
        message: String,
        ctx: Option<HashMap<String, String>>,
    ) {
        self.logs
            .add_error(None, SystemProcess::System, process_name, message, ctx);
    }

    fn write_debug_info(
        &self,
        process_name: String,
        message: String,
        ctx: Option<HashMap<String, String>>,
    ) {
        self.logs
            .add_error(None, SystemProcess::Debug, process_name, message, ctx);
    }
}
