use std::{
    sync::{
        atomic::{AtomicBool, AtomicI64},
        Arc,
    },
    time::Duration,
};

use my_no_sql_core::db::DbInstance;
use my_tcp_sockets::TcpClient;
use rust_extensions::{
    date_time::DateTimeAsMicroseconds, events_loop::EventsLoop, AppStates, Logger,
};

use crate::{
    data_readers::DataReadersList, db_operations::multipart::MultipartList, db_sync::SyncEvent,
    settings_reader::SettingsModel,
};

use super::{
    logs::{Logs, SystemProcess},
    PrometheusMetrics,
};

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
    pub sync: EventsLoop<SyncEvent>,
    pub states: Arc<AppStates>,
    pub tcp_client: TcpClient,

    pub master_node_ping_interval: AtomicI64,
    pub connected_to_main_node: AtomicBool,
}

impl AppContext {
    pub fn new(logs: Arc<Logs>, settings: Arc<SettingsModel>) -> Self {
        let tcp_client = TcpClient::new(
            "NodeConnection".to_string(),
            settings.main_server.to_string(),
        );
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
            sync: EventsLoop::new("SyncEventsLoop".to_string()),
            tcp_client,
            master_node_ping_interval: AtomicI64::new(0),
            connected_to_main_node: AtomicBool::new(false),
        }
    }
}

impl Logger for AppContext {
    fn write_info(&self, process_name: String, message: String, context: Option<String>) {
        self.logs
            .add_info(None, SystemProcess::System, process_name, message, context);
    }

    fn write_error(&self, process_name: String, message: String, context: Option<String>) {
        self.logs
            .add_fatal_error(None, SystemProcess::System, process_name, message, context);
    }

    fn write_warning(&self, process_name: String, message: String, ctx: Option<String>) {
        self.logs
            .add_error(None, SystemProcess::System, process_name, message, ctx);
    }

    fn write_fatal_error(&self, process_name: String, message: String, ctx: Option<String>) {
        self.logs
            .add_error(None, SystemProcess::System, process_name, message, ctx);
    }
}
