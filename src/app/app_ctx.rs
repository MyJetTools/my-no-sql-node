use std::{
    sync::{atomic::AtomicUsize, Arc},
    time::Duration,
};

use my_no_sql_core::db::DbInstance;
use rust_extensions::{
    date_time::DateTimeAsMicroseconds, events_loop::EventsLoop, AppStates, Logger,
};

use crate::{
    data_readers::DataReadersList, db_operations::multipart::MultipartList, db_sync::SyncEvent,
    persist::PersistMarkersByTable, settings_reader::SettingsModel,
};

use super::{
    logs::{Logs, SystemProcess},
    PrometheusMetrics,
};

pub const APP_VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub const DEFAULT_PERSIST_PERIOD: crate::db_sync::DataSynchronizationPeriod =
    crate::db_sync::DataSynchronizationPeriod::Sec5;

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
    pub persist_markers: PersistMarkersByTable,
    persist_amount: AtomicUsize,
}

impl AppContext {
    pub fn new(logs: Arc<Logs>, settings: Arc<SettingsModel>) -> Self {
        AppContext {
            persist_markers: PersistMarkersByTable::new(),
            created: DateTimeAsMicroseconds::now(),
            db: DbInstance::new(),
            logs,
            metrics: PrometheusMetrics::new(),
            process_id: uuid::Uuid::new_v4().to_string(),
            states: Arc::new(AppStates::create_un_initialized()),

            data_readers: DataReadersList::new(Duration::from_secs(30)),
            multipart_list: MultipartList::new(),

            settings,
            persist_amount: AtomicUsize::new(0),
            sync: EventsLoop::new("SyncEventsLoop".to_string()),
        }
    }

    pub fn update_persist_amount(&self, value: usize) {
        self.persist_amount
            .store(value, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn get_persist_amount(&self) -> usize {
        self.persist_amount
            .load(std::sync::atomic::Ordering::Relaxed)
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
