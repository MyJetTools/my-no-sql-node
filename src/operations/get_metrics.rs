use my_no_sql_sdk::server::DbTable;

pub struct DbTableMetrics {
    pub table_size: usize,
    pub partitions_amount: usize,
    pub records_amount: usize,
}

pub async fn get_table_metrics(db_table_wrapper: &DbTable) -> DbTableMetrics {
    let table_read_access = db_table_wrapper.data.read();

    DbTableMetrics {
        table_size: table_read_access.get_table_size(),
        partitions_amount: table_read_access.get_partitions_amount(),
        records_amount: table_read_access.get_rows_amount(),
    }
}
