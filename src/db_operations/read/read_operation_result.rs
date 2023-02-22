use std::{collections::HashMap, sync::Arc};

use my_json::json_writer::JsonArrayWriter;
use my_no_sql_core::db::DbRow;
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::app::AppContext;

#[derive(Clone, Debug)]
pub struct UpdateStatistics {
    pub update_partition_last_read_access_time: bool,
    pub update_rows_last_read_access_time: bool,
    pub update_partition_expiration_time: Option<Option<DateTimeAsMicroseconds>>,
    pub update_rows_expiration_time: Option<Option<DateTimeAsMicroseconds>>,
}

impl UpdateStatistics {
    pub fn has_statistics_to_update(&self) -> bool {
        self.update_partition_last_read_access_time
            || self.update_rows_last_read_access_time
            || self.update_partition_expiration_time.is_some()
            || self.update_rows_expiration_time.is_some()
    }

    pub async fn update_statistics(
        &self,
        app: &Arc<AppContext>,
        table_name: &str,
        partition_key: &String,
        db_rows: &[&Arc<DbRow>],
    ) {
        if self.update_partition_last_read_access_time {
            app.sync_to_main_node_queue
                .update_partitions_last_read_time(table_name, [partition_key].into_iter())
                .await;
        }

        if self.update_rows_last_read_access_time {
            app.sync_to_main_node_queue
                .update_rows_last_read_time(
                    table_name,
                    partition_key,
                    db_rows.iter().map(|db_row| &db_row.row_key),
                )
                .await;
        }

        if let Some(update_partition_expiration_time) = self.update_partition_expiration_time {
            app.sync_to_main_node_queue
                .update_partition_expiration_time(
                    table_name,
                    partition_key,
                    update_partition_expiration_time,
                )
                .await;
        }

        if let Some(update_rows_expiration_time) = self.update_rows_expiration_time {
            app.sync_to_main_node_queue
                .update_rows_expiration_time(
                    table_name,
                    partition_key,
                    db_rows.iter().map(|db_row| &db_row.row_key),
                    update_rows_expiration_time,
                )
                .await;
        }
    }
}

pub enum ReadOperationResult {
    SingleRow(Vec<u8>),
    RowsArray(Vec<u8>),
    EmptyArray,
}

impl ReadOperationResult {
    pub async fn compile_array_or_empty(
        app: &Arc<AppContext>,
        table_name: &str,
        db_rows: Option<HashMap<String, Vec<&Arc<DbRow>>>>,
        update_statistics: UpdateStatistics,
    ) -> Self {
        if db_rows.is_none() {
            return Self::EmptyArray;
        }

        let mut json_array_writer = JsonArrayWriter::new();

        let db_rows = db_rows.unwrap();

        if update_statistics.has_statistics_to_update() {
            for (partition_key, db_rows) in &db_rows {
                update_statistics
                    .update_statistics(app, table_name, partition_key, db_rows)
                    .await;
            }
        }

        for (_, db_rows) in db_rows {
            for db_row in db_rows {
                json_array_writer.write_raw_element(&db_row.data);
            }
        }

        return ReadOperationResult::RowsArray(json_array_writer.build());
    }

    pub async fn compile_array_or_empty_from_partition(
        app: &Arc<AppContext>,
        table_name: &str,
        partition_key: &String,
        db_rows: Option<Vec<&Arc<DbRow>>>,
        update_statistics: UpdateStatistics,
    ) -> Self {
        if db_rows.is_none() {
            return Self::EmptyArray;
        }

        let mut json_array_writer = JsonArrayWriter::new();

        let db_rows = db_rows.unwrap();

        update_statistics
            .update_statistics(app, table_name, partition_key, &db_rows)
            .await;

        for db_row in db_rows {
            json_array_writer.write_raw_element(&db_row.data);
        }

        return ReadOperationResult::RowsArray(json_array_writer.build());
    }
}
