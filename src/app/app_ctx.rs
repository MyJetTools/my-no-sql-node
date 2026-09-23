use std::{sync::Arc, time::Duration};

use my_no_sql_sdk::core::{
    db::DbNamespaceName,
    rust_extensions::{events_loop::EventsLoop, AppStates},
};

use crate::{
    data_readers::DataReadersList,
    main_server_http::MainServerHttp,
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
    /// Where the writes go - `None` when the node is not told the main node's HTTP api.
    pub main_server_http: Option<MainServerHttp>,
}

impl AppContext {
    pub fn new(settings: Arc<SettingsModel>) -> Self {
        Self {
            namespaces: NodeNamespaces::new(settings.get_max_namespaces()),
            data_readers: DataReadersList::new(HTTP_SESSION_TIMEOUT),
            metrics: PrometheusMetrics::new(),
            sync_to_clients: EventsLoop::new("SyncToClients"),
            states: Arc::new(AppStates::create_initialized()),
            main_server_http: settings.main_server_http.as_ref().map(|url| {
                MainServerHttp::new(
                    url.to_string(),
                    settings.get_compress_writes(),
                    settings.get_main_server_http_timeout(),
                )
            }),
            settings,
        }
    }

    pub fn dispatch(&self, namespace: DbNamespaceName, sync_event: SyncEvent) {
        self.sync_to_clients
            .send(NamespaceSyncEvent::new(namespace, sync_event));
    }
}
