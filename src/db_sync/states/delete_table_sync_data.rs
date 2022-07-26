use my_no_sql_core::db::DbTableInner;

use crate::db_sync::EventSource;

use super::SyncTableData;

pub struct DeleteTableSyncData {
    pub table_data: SyncTableData,
    pub event_src: EventSource,
}

impl DeleteTableSyncData {
    pub fn new(db_table_data: &DbTableInner, event_src: EventSource, persist: bool) -> Self {
        Self {
            table_data: SyncTableData::new(db_table_data, persist),
            event_src,
        }
    }
}
