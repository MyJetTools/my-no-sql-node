use my_no_sql_sdk::{
    core::db::{DbTableInner, DbTableName},
    server::db_snapshots::DbRowsByPartitionsSnapshot,
};

pub struct UpdateRowsSyncData {
    pub table_name: DbTableName,

    pub rows_by_partition: DbRowsByPartitionsSnapshot,
}

impl UpdateRowsSyncData {
    pub fn new(db_table: &DbTableInner) -> Self {
        Self {
            table_name: db_table.name.clone(),
            rows_by_partition: DbRowsByPartitionsSnapshot::new(),
        }
    }
}
