use std::{sync::Arc, time::Duration};

use my_no_sql_sdk::core::{
    db::DbNamespaceName,
    rust_extensions::{events_loop::EventsLoop, AppStates},
};

use crate::{
    data_readers::DataReadersList,
    db_sync::{NamespaceSyncEvent, SyncEvent},
    namespaces::NodeNamespaces,
    settings_reader::SettingsModel,
};

use super::PrometheusMetrics;

pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const HTTP_SESSION_TIMEOUT: Duration = Duration::from_secs(30);

pub struct AppContext {
    pub namespaces: NodeNamespaces,
    pub data_readers: DataReadersList,
    pub metrics: PrometheusMetrics,
    pub sync_to_clients: EventsLoop<NamespaceSyncEvent>,
    pub states: Arc<AppStates>,
    pub settings: Arc<SettingsModel>,
}

impl AppContext {
    pub fn new(settings: Arc<SettingsModel>) -> Self {
        Self {
            namespaces: NodeNamespaces::new(settings.get_max_namespaces()),
            data_readers: DataReadersList::new(HTTP_SESSION_TIMEOUT),
            metrics: PrometheusMetrics::new(),
            sync_to_clients: EventsLoop::new("SyncToClients"),
            states: Arc::new(AppStates::create_initialized()),
            settings,
        }
    }

    pub fn dispatch(&self, namespace: DbNamespaceName, sync_event: SyncEvent) {
        self.sync_to_clients
            .send(NamespaceSyncEvent::new(namespace, sync_event));
    }
}
