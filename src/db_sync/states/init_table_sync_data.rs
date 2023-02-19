use std::sync::Arc;

use my_no_sql_server_core::DbTableWrapper;

use crate::db_sync::EventSource;

pub struct InitTableEventSyncData {
    pub db_table: Arc<DbTableWrapper>,
    pub event_src: EventSource,
}

impl InitTableEventSyncData {
    pub fn new(db_table_wrapper: Arc<DbTableWrapper>, event_src: EventSource) -> Self {
        Self {
            db_table: db_table_wrapper,
            event_src,
        }
    }
}
