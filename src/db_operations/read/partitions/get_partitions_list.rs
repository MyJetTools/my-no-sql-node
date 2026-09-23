use my_no_sql_sdk::server::DbTable;

pub struct PartitionsList {
    pub total_amount: usize,
    pub partition_keys: Vec<String>,
}

pub fn get_partitions(
    db_table: &DbTable,
    limit: Option<usize>,
    skip: Option<usize>,
) -> PartitionsList {
    let table_data = db_table.data.read();

    let partition_keys =
        crate::db_operations::read::filter_it(table_data.partitions.get_partitions(), limit, skip)
            .iter()
            .map(|itm| itm.partition_key.to_string())
            .collect();

    PartitionsList {
        total_amount: table_data.partitions.len(),
        partition_keys,
    }
}
