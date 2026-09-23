use my_no_sql_sdk::{core::db::DbTableName, server::db_snapshots::DbRowsByPartitionsSnapshot};

pub struct UpdateRowsSyncData {
    pub table_name: DbTableName,
    pub rows_by_partition: DbRowsByPartitionsSnapshot,
}

impl UpdateRowsSyncData {
    pub fn new(table_name: DbTableName) -> Self {
        Self {
            table_name,
            rows_by_partition: DbRowsByPartitionsSnapshot::new(),
        }
    }
}
