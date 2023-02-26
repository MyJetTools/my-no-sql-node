use my_no_sql_server_core::DbTableWrapper;

pub struct DbTableMetrics {
    pub table_size: usize,
    pub partitions_amount: usize,
    pub records_amount: usize,
}

pub async fn get_table_metrics(db_table_wrapper: &DbTableWrapper) -> DbTableMetrics {
    let table_read_access = db_table_wrapper.data.read().await;

    DbTableMetrics {
        table_size: table_read_access.get_table_size(),
        partitions_amount: table_read_access.get_partitions_amount(),
        records_amount: table_read_access.get_rows_amount(),
    }
}
