use my_no_sql_sdk::server::DbTable;

/// The most partitions one request gets - the details are built under the table's read lock.
pub const MAX_PARTITIONS_DETAILS: usize = 1000;

pub struct PartitionDetails {
    pub partition_key: String,
    pub records_count: usize,
    pub data_size: usize,
}

pub struct PartitionsDetails {
    pub total: usize,
    pub matched: usize,
    pub partitions: Vec<PartitionDetails>,
}

/// The partitions whose key contains `filter` (case-insensitive), with their metrics, in the
/// table's partition order - `limit` of them at most, and never more than
/// `MAX_PARTITIONS_DETAILS`. `matched` counts every matching one.
pub fn get_partitions_details(
    db_table: &DbTable,
    filter: Option<&str>,
    limit: Option<usize>,
) -> PartitionsDetails {
    let limit = limit
        .unwrap_or(MAX_PARTITIONS_DETAILS)
        .min(MAX_PARTITIONS_DETAILS);

    let filter = filter
        .map(|itm| itm.trim().to_lowercase())
        .filter(|itm| !itm.is_empty());

    let table_data = db_table.data.read();

    let mut result = PartitionsDetails {
        total: table_data.partitions.len(),
        matched: 0,
        partitions: Vec::new(),
    };

    for partition in table_data.partitions.get_partitions() {
        let partition_key = partition.partition_key.to_string();

        if let Some(filter) = filter.as_ref() {
            if !partition_key.to_lowercase().contains(filter.as_str()) {
                continue;
            }
        }

        result.matched += 1;

        if result.partitions.len() < limit {
            result.partitions.push(PartitionDetails {
                partition_key,
                records_count: partition.rows_count(),
                data_size: partition.get_content_size(),
            });
        }
    }

    result
}
