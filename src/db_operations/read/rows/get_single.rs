use my_no_sql_core::db::{DbTable, UpdateExpirationTimeModel, UpdatePartitionReadMoment};
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{app::AppContext, db_operations::DbOperationError};

use super::super::ReadOperationResult;

pub async fn get_single(
    app: &AppContext,
    table: &DbTable,
    partition_key: &str,
    row_key: &str,
    update_expiration_time: Option<UpdateExpirationTimeModel>,
) -> Result<ReadOperationResult, DbOperationError> {
    super::super::super::check_app_states(app)?;

    if let Some(update_expiration_time) = update_expiration_time {
        get_single_and_update_expiration_time(
            table,
            partition_key,
            row_key,
            &update_expiration_time,
        )
        .await
    } else {
        get_single_and_update_no_expiration_time(table, partition_key, row_key).await
    }
}

async fn get_single_and_update_no_expiration_time(
    table: &DbTable,
    partition_key: &str,
    row_key: &str,
) -> Result<ReadOperationResult, DbOperationError> {
    let now = DateTimeAsMicroseconds::now();

    let table_data = table.data.read().await;

    table_data.last_read_time.update(now);

    let partition = table_data.get_partition(partition_key);

    if partition.is_none() {
        return Err(DbOperationError::RecordNotFound);
    }

    let partition = partition.unwrap();

    let db_row = partition.get_row(
        row_key,
        UpdatePartitionReadMoment::UpdateIfElementIsFound(now),
    );

    if db_row.is_none() {
        return Err(DbOperationError::RecordNotFound);
    }

    let db_row = db_row.unwrap();
    db_row.last_read_access.update(now);

    return Ok(ReadOperationResult::SingleRow(db_row.data.clone()));
}

async fn get_single_and_update_expiration_time(
    table: &DbTable,
    partition_key: &str,
    row_key: &str,
    update_expiration_time: &UpdateExpirationTimeModel,
) -> Result<ReadOperationResult, DbOperationError> {
    let now = DateTimeAsMicroseconds::now();

    let mut table_data = table.data.write().await;

    table_data.last_read_time.update(now);

    let partition = table_data.get_partition_mut(partition_key);

    if partition.is_none() {
        return Err(DbOperationError::RecordNotFound);
    }

    let partition = partition.unwrap();

    let db_row = partition.get_row_and_update_expiration_time(
        row_key,
        UpdatePartitionReadMoment::UpdateIfElementIsFound(now),
        update_expiration_time,
    );

    if db_row.is_none() {
        return Err(DbOperationError::RecordNotFound);
    }

    let db_row = db_row.unwrap();
    db_row.last_read_access.update(now);

    return Ok(ReadOperationResult::SingleRow(db_row.data.clone()));
}
