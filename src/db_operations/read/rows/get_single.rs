use my_no_sql_server_core::DbTableWrapper;

use crate::{
    app::AppContext,
    db_operations::{read::read_operation_result::UpdateStatistics, DbOperationError},
};

use super::super::ReadOperationResult;

pub async fn get_single(
    app: &std::sync::Arc<AppContext>,
    db_table_wrapper: &DbTableWrapper,
    partition_key: &String,
    row_key: &str,
    update_statistics: UpdateStatistics,
) -> Result<ReadOperationResult, DbOperationError> {
    super::super::super::check_app_states(app)?;

    let mut db_table = db_table_wrapper.data.write().await;

    let partition = db_table.get_partition_mut(partition_key);

    if partition.is_none() {
        return Err(DbOperationError::RecordNotFound);
    }

    let partition = partition.unwrap();

    let db_row = partition.get_row(row_key);

    if db_row.is_none() {
        return Err(DbOperationError::RecordNotFound);
    }

    let db_row = db_row.unwrap();

    update_statistics
        .update_statistics(app, &db_table_wrapper.name, partition_key, &[db_row])
        .await;

    return Ok(ReadOperationResult::SingleRow(db_row.data.clone()));
}
