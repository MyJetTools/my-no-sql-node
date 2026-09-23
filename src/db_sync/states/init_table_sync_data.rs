use std::sync::Arc;

use my_no_sql_sdk::server::DbTable;

/// The whole table for every reader subscribed to it. The rows are serialized when the event
/// is delivered, so a reader always gets the latest state of the table.
pub struct InitTableEventSyncData {
    pub db_table: Arc<DbTable>,
}

impl InitTableEventSyncData {
    pub fn new(db_table: Arc<DbTable>) -> Self {
        Self { db_table }
    }
}
