use std::sync::Arc;

use my_no_sql_sdk::server::DbTable;

pub struct InitTableEventSyncData {
    pub db_table: Arc<DbTable>,
}

impl InitTableEventSyncData {
    pub fn new(db_table_wrapper: Arc<DbTable>) -> Self {
        Self {
            db_table: db_table_wrapper,
        }
    }
}
