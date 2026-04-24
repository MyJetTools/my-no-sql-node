use std::collections::BTreeMap;

use my_json::json_writer::{JsonNullValue, JsonObjectWriter};
use my_no_sql_sdk::{
    core::db::{DbTableInner, DbTableName},
    server::db_snapshots::DbPartitionSnapshot,
};

pub struct InitPartitionsSyncData {
    pub table_name: DbTableName,
    pub partitions_to_update: BTreeMap<String, Option<DbPartitionSnapshot>>,
}

impl InitPartitionsSyncData {
    pub fn new_as_update_partition(db_table: &DbTableInner, partition_key: &str) -> Self {
        let mut partitions_to_update = BTreeMap::new();

        if let Some(db_partition) = db_table.get_partition(partition_key) {
            let partition_snapshot: DbPartitionSnapshot = db_partition.into();
            partitions_to_update.insert(partition_key.to_string(), Some(partition_snapshot));
        } else {
            partitions_to_update.insert(partition_key.to_string(), None);
        }

        Self {
            table_name: db_table.name.clone(),
            partitions_to_update,
        }
    }

    pub fn as_json(&self) -> JsonObjectWriter {
        let mut json_object_writer = JsonObjectWriter::new();

        for (partition_key, db_partition) in &self.partitions_to_update {
            if let Some(db_partition_snapshot) = db_partition {
                json_object_writer = json_object_writer.write(
                    partition_key,
                    db_partition_snapshot.db_rows_snapshot.as_json_array(),
                );
            } else {
                json_object_writer = json_object_writer.write(partition_key, JsonNullValue);
            }
        }

        json_object_writer
    }
}
