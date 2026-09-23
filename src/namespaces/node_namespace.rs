use std::sync::Arc;

use my_no_sql_sdk::{
    core::{db::DbNamespaceName, rust_extensions::AppStates},
    server::DbInstance,
};

use crate::settings_reader::SettingsModel;

use super::MainNodeConnection;

/// Everything the node holds for one namespace: the tables replicated from it and the
/// connection they are replicated over. Namespaces share nothing - a table of one namespace is
/// invisible to the readers of every other one.
pub struct NodeNamespace {
    pub name: DbNamespaceName,
    pub db: DbInstance,
    pub main_node: MainNodeConnection,
}

impl NodeNamespace {
    pub fn new(
        name: DbNamespaceName,
        settings: &SettingsModel,
        app_states: &Arc<AppStates>,
    ) -> Self {
        Self {
            main_node: MainNodeConnection::new(&name, settings.main_server.as_str(), app_states),
            db: DbInstance::new(),
            name,
        }
    }
}
