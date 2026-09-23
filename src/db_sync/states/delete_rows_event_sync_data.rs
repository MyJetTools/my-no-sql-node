use std::{collections::BTreeMap, sync::Arc};

use my_json::json_writer::{JsonArrayWriter, JsonObjectWriter};
use my_no_sql_sdk::core::db::{DbRow, DbTableName};

pub struct DeleteRowsEventSyncData {
    pub table_name: DbTableName,
    /// Deleted rows by partition key, then by row key.
    pub deleted_rows: BTreeMap<String, BTreeMap<String, Arc<DbRow>>>,
}

impl DeleteRowsEventSyncData {
    pub fn new(table_name: DbTableName) -> Self {
        Self {
            table_name,
            deleted_rows: BTreeMap::new(),
        }
    }

    pub fn add_deleted_row(&mut self, partition_key: &str, deleted_row: Arc<DbRow>) {
        self.deleted_rows
            .entry(partition_key.to_string())
            .or_default()
            .insert(deleted_row.get_row_key().to_string(), deleted_row);
    }

    pub fn has_data(&self) -> bool {
        !self.deleted_rows.is_empty()
    }

    /// `{"partition_key": ["row_key", ...], ...}` - the shape an HTTP reader expects.
    pub fn as_json(&self) -> JsonObjectWriter {
        let mut json_object_writer = JsonObjectWriter::new();

        for (partition_key, deleted_rows) in &self.deleted_rows {
            let mut row_keys = JsonArrayWriter::new();

            for row_key in deleted_rows.keys() {
                row_keys = row_keys.write(row_key.as_str());
            }

            json_object_writer = json_object_writer.write(partition_key, row_keys);
        }

        json_object_writer
    }
}
