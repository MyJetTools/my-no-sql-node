use my_no_sql_sdk::core::db::DbNamespaceName;

use super::SyncEvent;

/// A change together with the namespace it happened in. Readers are routed by
/// `(namespace, table)`: two namespaces may each hold a table of the same name, and a reader of
/// one of them must never see the other's rows.
pub struct NamespaceSyncEvent {
    pub namespace: DbNamespaceName,
    pub event: SyncEvent,
}

impl NamespaceSyncEvent {
    pub fn new(namespace: DbNamespaceName, event: SyncEvent) -> Self {
        Self { namespace, event }
    }
}
