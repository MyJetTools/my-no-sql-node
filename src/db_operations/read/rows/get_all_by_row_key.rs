use my_no_sql_core::db::{DbTable, UpdateExpirationTimeModel, UpdatePartitionReadMoment};
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{app::AppContext, db_operations::DbOperationError};

use super::super::{read_filter, ReadOperationResult};

pub async fn get_all_by_row_key(
    app: &AppContext,
    table: &DbTable,
    row_key: &str,
    limit: Option<usize>,
    skip: Option<usize>,
    update_expiration_time: Option<UpdateExpirationTimeModel>,
) -> Result<ReadOperationResult, DbOperationError> {
    super::super::super::check_app_states(app)?;

    let result = if let Some(update_expiration_time) = update_expiration_time {
        get_all_by_row_key_and_update_expiration_time(
            table,
            row_key,
            limit,
            skip,
            &update_expiration_time,
        )
        .await
    } else {
        get_all_by_row_key_and_update_no_expiration_time(table, row_key, limit, skip).await
    };

    Ok(result)
}

async fn get_all_by_row_key_and_update_no_expiration_time(
    table: &DbTable,
    row_key: &str,
    limit: Option<usize>,
    skip: Option<usize>,
) -> ReadOperationResult {
    let now = DateTimeAsMicroseconds::now();

    let table_data = table.data.read().await;

    table_data.last_read_time.update(now);

    let mut result = Vec::new();

    let now = DateTimeAsMicroseconds::now();

    for partition in table_data.get_partitions() {
        let get_row_result = partition.get_row(
            row_key,
            UpdatePartitionReadMoment::UpdateIfElementIsFound(now),
        );

        if let Some(db_row) = get_row_result {
            result.push(db_row);
        }
    }

    return ReadOperationResult::RowsArray(read_filter::filter_it(
        result.into_iter(),
        limit,
        skip,
        now,
    ));
}

async fn get_all_by_row_key_and_update_expiration_time(
    table: &DbTable,
    row_key: &str,
    limit: Option<usize>,
    skip: Option<usize>,
    update_expiration_time: &UpdateExpirationTimeModel,
) -> ReadOperationResult {
    let now = DateTimeAsMicroseconds::now();

    let mut table_data = table.data.write().await;

    table_data.last_read_time.update(now);

    let mut result = Vec::new();

    let now = DateTimeAsMicroseconds::now();

    for partition in table_data.partitions.values_mut() {
        let get_row_result = partition.get_row_and_update_expiration_time(
            row_key,
            UpdatePartitionReadMoment::UpdateIfElementIsFound(now),
            update_expiration_time,
        );

        if let Some(db_row) = get_row_result {
            result.push(db_row);
        }
    }

    return ReadOperationResult::RowsArray(read_filter::filter_it(
        result.iter().map(|itm| itm),
        limit,
        skip,
        now,
    ));
}
