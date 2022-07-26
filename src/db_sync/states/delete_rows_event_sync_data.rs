use std::{collections::BTreeMap, sync::Arc};

use my_json::json_writer::{JsonArrayWriter, JsonObjectWriter};
use my_no_sql_core::db::{DbRow, DbTableInner};

use crate::db_sync::EventSource;

use super::SyncTableData;

pub struct DeleteRowsEventSyncData {
    pub table_data: SyncTableData,
    pub event_src: EventSource,
    pub deleted_partitions: Option<BTreeMap<String, ()>>,
    pub deleted_rows: Option<BTreeMap<String, BTreeMap<String, Arc<DbRow>>>>,
}

impl DeleteRowsEventSyncData {
    pub fn new(table_data: &DbTableInner, persist: bool, event_src: EventSource) -> Self {
        Self {
            table_data: SyncTableData::new(table_data, persist),
            event_src,
            deleted_partitions: None,
            deleted_rows: None,
        }
    }

    fn check_that_we_are_in_partition_mode(
        &mut self,
        partition_key: &str,
    ) -> &mut BTreeMap<String, BTreeMap<String, Arc<DbRow>>> {
        if let Some(deleted_partitions) = &self.deleted_partitions {
            if deleted_partitions.contains_key(partition_key) {
                panic!("Can not add deleted rows from partition {}", partition_key);
            }
        }

        if self.deleted_rows.is_none() {
            self.deleted_rows = Some(BTreeMap::new())
        }

        return self.deleted_rows.as_mut().unwrap();
    }

    pub fn add_deleted_row(&mut self, partition_key: &str, deleted_row: Arc<DbRow>) {
        let deleted_rows_btree_map = self.check_that_we_are_in_partition_mode(partition_key);

        if !deleted_rows_btree_map.contains_key(partition_key) {
            deleted_rows_btree_map.insert(partition_key.to_string(), BTreeMap::new());
        }

        deleted_rows_btree_map
            .get_mut(partition_key)
            .unwrap()
            .insert(deleted_row.row_key.to_string(), deleted_row.clone());
    }

    pub fn add_deleted_rows(&mut self, partition_key: &str, deleted_rows: &[Arc<DbRow>]) {
        let deleted_rows_btree_map = self.check_that_we_are_in_partition_mode(partition_key);

        if !deleted_rows_btree_map.contains_key(partition_key) {
            deleted_rows_btree_map.insert(partition_key.to_string(), BTreeMap::new());
        }

        let by_partition = deleted_rows_btree_map.get_mut(partition_key).unwrap();

        for deleted_row in deleted_rows {
            by_partition.insert(deleted_row.row_key.to_string(), deleted_row.clone());
        }
    }

    pub fn new_deleted_partition(&mut self, partition_key: String) {
        if let Some(deleted_rows) = &mut self.deleted_rows {
            deleted_rows.remove(partition_key.as_str());
        }

        if self.deleted_partitions.is_none() {
            self.deleted_partitions = Some(BTreeMap::new());
        }

        self.deleted_partitions
            .as_mut()
            .unwrap()
            .insert(partition_key, ());
    }

    pub fn as_vec(&self) -> Vec<u8> {
        let mut json_object_writer = JsonObjectWriter::new();

        {
            if let Some(deleted_partitions) = &self.deleted_partitions {
                for partition_key in deleted_partitions.keys() {
                    json_object_writer.write_null_value(partition_key);
                }
            }

            if let Some(deleted_rows) = &self.deleted_rows {
                for (partition_key, deleted_rows) in deleted_rows {
                    let mut deleted_rows_json_array = JsonArrayWriter::new();
                    for deleted_row in deleted_rows.values() {
                        deleted_rows_json_array.write_string_element(deleted_row.row_key.as_str());
                    }
                    json_object_writer.write_object(partition_key, deleted_rows_json_array);
                }
            }
        }

        json_object_writer.build()
    }
}
