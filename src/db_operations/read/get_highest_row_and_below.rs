use my_json::json_writer::JsonArrayWriter;
use my_no_sql_core::db::{DbTable, UpdateExpirationTimeModel};
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{app::AppContext, db_operations::DbOperationError};

use super::ReadOperationResult;

pub async fn get_highest_row_and_below(
    app: &AppContext,
    db_table: &DbTable,
    partition_key: &str,
    row_key: &String,
    limit: Option<usize>,
    update_expiration_time: Option<UpdateExpirationTimeModel>,
) -> Result<ReadOperationResult, DbOperationError> {
    super::super::check_app_states(app)?;

    let result = if let Some(update_expiration_time) = update_expiration_time {
        get_highest_row_and_below_and_update_expiration_time(
            db_table,
            partition_key,
            row_key,
            limit,
            update_expiration_time,
        )
        .await
    } else {
        get_highest_row_and_below_with_no_expiration_time_update(
            db_table,
            partition_key,
            row_key,
            limit,
        )
        .await
    };

    Ok(result)
}

async fn get_highest_row_and_below_with_no_expiration_time_update(
    db_table: &DbTable,
    partition_key: &str,
    row_key: &String,
    limit: Option<usize>,
) -> ReadOperationResult {
    let now = DateTimeAsMicroseconds::now();

    let read_access = db_table.data.read().await;

    let db_partition = read_access.get_partition(partition_key);

    if db_partition.is_none() {
        return ReadOperationResult::EmptyArray;
    }

    let db_partition = db_partition.unwrap();

    let db_rows = db_partition.get_highest_row_and_below(row_key, Some(now), limit);

    if db_rows.len() == 0 {
        return ReadOperationResult::EmptyArray;
    }

    let mut json_array_writer = JsonArrayWriter::new();

    for db_row in db_rows {
        json_array_writer.write_raw_element(db_row.data.as_ref());
    }

    return ReadOperationResult::RowsArray(json_array_writer.build());
}

async fn get_highest_row_and_below_and_update_expiration_time(
    db_table: &DbTable,
    partition_key: &str,
    row_key: &String,
    limit: Option<usize>,
    update_expiration_time: UpdateExpirationTimeModel,
) -> ReadOperationResult {
    let now = DateTimeAsMicroseconds::now();

    let mut wrtite_access = db_table.data.write().await;

    let db_partition = wrtite_access.get_partition_mut(partition_key);

    if db_partition.is_none() {
        return ReadOperationResult::EmptyArray;
    }

    let db_partition = db_partition.unwrap();

    let db_rows = db_partition.get_highest_row_and_below_and_update_expiration_time(
        row_key,
        Some(now),
        limit,
        &update_expiration_time,
    );

    if db_rows.len() == 0 {
        return ReadOperationResult::EmptyArray;
    }

    let mut json_array_writer = JsonArrayWriter::new();

    for db_row in db_rows {
        json_array_writer.write_raw_element(db_row.data.as_ref());
    }

    return ReadOperationResult::RowsArray(json_array_writer.build());
}
