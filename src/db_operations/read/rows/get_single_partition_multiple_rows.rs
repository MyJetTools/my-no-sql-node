use my_json::json_writer::JsonArrayWriter;
use my_no_sql_core::db::{DbTable, UpdateExpirationTimeModel, UpdatePartitionReadMoment};
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{app::AppContext, db_operations::DbOperationError};

use super::super::ReadOperationResult;

pub async fn get_single_partition_multiple_rows(
    app: &AppContext,
    table: &DbTable,
    partition_key: &str,
    row_keys: Vec<String>,
    update_expiration_time: Option<UpdateExpirationTimeModel>,
) -> Result<ReadOperationResult, DbOperationError> {
    super::super::super::check_app_states(app)?;

    let result = if let Some(update_expiration_time) = update_expiration_time {
        get_single_partition_multiple_rows_with_expiration_time_update(
            table,
            partition_key,
            row_keys,
            update_expiration_time,
        )
        .await
    } else {
        get_single_partition_multiple_rows_with_no_expiration_time_update(
            table,
            partition_key,
            row_keys,
        )
        .await
    };

    Ok(result)
}

pub async fn get_single_partition_multiple_rows_with_no_expiration_time_update(
    table: &DbTable,
    partition_key: &str,
    row_keys: Vec<String>,
) -> ReadOperationResult {
    let now = DateTimeAsMicroseconds::now();
    let read_access = table.data.read().await;

    let db_partition = read_access.get_partition(partition_key);

    if db_partition.is_none() {
        return ReadOperationResult::EmptyArray;
    }

    let db_partition = db_partition.unwrap();

    let mut json_array_writer = JsonArrayWriter::new();

    for row_key in &row_keys {
        let db_row = db_partition.get_row(row_key, UpdatePartitionReadMoment::None);

        if let Some(db_row) = db_row {
            db_row.update_last_access(now);
            json_array_writer.write_raw_element(&db_row.data);
        }
    }

    return ReadOperationResult::RowsArray(json_array_writer.build());
}

pub async fn get_single_partition_multiple_rows_with_expiration_time_update(
    table: &DbTable,
    partition_key: &str,
    row_keys: Vec<String>,
    update_expiration_time: UpdateExpirationTimeModel,
) -> ReadOperationResult {
    let now = DateTimeAsMicroseconds::now();
    let mut write_access = table.data.write().await;

    let db_partition = write_access.get_partition_mut(partition_key);

    if db_partition.is_none() {
        return ReadOperationResult::EmptyArray;
    }

    let db_partition = db_partition.unwrap();

    let mut json_array_writer = JsonArrayWriter::new();

    for row_key in &row_keys {
        let db_row = db_partition.get_row_and_update_expiration_time(
            row_key,
            UpdatePartitionReadMoment::None,
            &update_expiration_time,
        );

        if let Some(db_row) = db_row {
            db_row.update_last_access(now);
            json_array_writer.write_raw_element(&db_row.data);
        }
    }

    return ReadOperationResult::RowsArray(json_array_writer.build());
}
