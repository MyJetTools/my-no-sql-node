use my_no_sql_core::db::DbTable;
use rust_extensions::date_time::DateTimeAsMicroseconds;

pub struct DbTableMetrics {
    pub table_size: usize,
    pub partitions_amount: usize,
    pub expiration_index_records_amount: usize,
    pub records_amount: usize,
    pub last_update_time: DateTimeAsMicroseconds,
}

pub async fn get_table_metrics(db_table: &DbTable) -> DbTableMetrics {
    let table_read_access = db_table.data.read().await;

    DbTableMetrics {
        table_size: table_read_access.get_table_size(),
        partitions_amount: table_read_access.get_partitions_amount(),
        expiration_index_records_amount: table_read_access.get_expiration_index_rows_amount(),
        records_amount: table_read_access.get_rows_amount(),
        last_update_time: table_read_access.get_last_update_time(),
    }
}
