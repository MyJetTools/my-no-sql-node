use my_no_sql_core::db::DbTable;
use my_no_sql_server_core::db_snapshots::DbRowsByPartitionsSnapshot;

use crate::db_sync::EventSource;

pub struct UpdateRowsSyncData {
    pub table_name: String,
    pub event_src: EventSource,
    pub rows_by_partition: DbRowsByPartitionsSnapshot,
}

impl UpdateRowsSyncData {
    pub fn new(db_table: &DbTable, event_src: EventSource) -> Self {
        Self {
            table_name: db_table.name.clone(),
            event_src,
            rows_by_partition: DbRowsByPartitionsSnapshot::new(),
        }
    }
}
