use std::collections::BTreeMap;

use my_json::json_writer::{JsonNullValue, JsonObjectWriter};
use my_no_sql_sdk::{
    core::db::{DbTableInner, DbTableName},
    server::db_snapshots::DbPartitionSnapshot,
};

pub struct InitPartitionsSyncData {
    pub table_name: DbTableName,
    /// `None` - the partition is gone.
    pub partitions_to_update: BTreeMap<String, Option<DbPartitionSnapshot>>,
}

impl InitPartitionsSyncData {
    pub fn new_as_update_partition(db_table: &DbTableInner, partition_key: &str) -> Self {
        let mut partitions_to_update = BTreeMap::new();

        partitions_to_update.insert(
            partition_key.to_string(),
            db_table
                .get_partition(partition_key)
                .map(|db_partition| db_partition.into()),
        );

        Self {
            table_name: db_table.name.clone(),
            partitions_to_update,
        }
    }

    pub fn as_json(&self) -> JsonObjectWriter {
        let mut json_object_writer = JsonObjectWriter::new();

        for (partition_key, db_partition) in &self.partitions_to_update {
            json_object_writer = match db_partition {
                Some(db_partition_snapshot) => json_object_writer.write(
                    partition_key,
                    db_partition_snapshot.db_rows_snapshot.as_json_array(),
                ),
                None => json_object_writer.write(partition_key, JsonNullValue),
            };
        }

        json_object_writer
    }
}
