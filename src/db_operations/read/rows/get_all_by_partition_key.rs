use my_no_sql_core::db::{DbTable, UpdateExpirationTimeModel};
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{app::AppContext, db_operations::DbOperationError};

use super::super::{read_filter, ReadOperationResult};

pub async fn get_all_by_partition_key(
    app: &AppContext,
    table: &DbTable,
    partition_key: &str,
    limit: Option<usize>,
    skip: Option<usize>,
    update_expiration_time: Option<UpdateExpirationTimeModel>,
) -> Result<ReadOperationResult, DbOperationError> {
    super::super::super::check_app_states(app)?;

    let result = if let Some(update_expiration_time) = update_expiration_time {
        get_all_by_partition_key_and_update_expiration_time(
            table,
            partition_key,
            limit,
            skip,
            &update_expiration_time,
        )
        .await
    } else {
        get_all_by_partition_key_and_no_updates(table, partition_key, limit, skip).await
    };

    Ok(result)
}

pub async fn get_all_by_partition_key_and_no_updates(
    table: &DbTable,
    partition_key: &str,
    limit: Option<usize>,
    skip: Option<usize>,
) -> ReadOperationResult {
    let now = DateTimeAsMicroseconds::now();

    let table_data = table.data.read().await;

    table_data.last_read_time.update(now);

    let get_partition_result = table_data.get_partition(partition_key);

    match get_partition_result {
        Some(partition) => ReadOperationResult::RowsArray(read_filter::filter_it(
            partition.get_all_rows(Some(now)),
            limit,
            skip,
            now,
        )),
        None => ReadOperationResult::EmptyArray,
    }
}

pub async fn get_all_by_partition_key_and_update_expiration_time(
    table: &DbTable,
    partition_key: &str,
    limit: Option<usize>,
    skip: Option<usize>,
    update_expiration_time: &UpdateExpirationTimeModel,
) -> ReadOperationResult {
    let now = DateTimeAsMicroseconds::now();

    let mut table_data = table.data.write().await;

    table_data.last_read_time.update(now);

    let get_partition_result = table_data.get_partition_mut(partition_key);

    match get_partition_result {
        Some(partition) => ReadOperationResult::RowsArray(read_filter::filter_it(
            partition
                .get_all_rows_and_update_expiration_time(Some(now), update_expiration_time)
                .iter()
                .map(|itm| itm),
            limit,
            skip,
            now,
        )),
        None => ReadOperationResult::EmptyArray,
    }
}
