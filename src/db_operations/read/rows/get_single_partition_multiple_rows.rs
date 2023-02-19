use my_no_sql_server_core::DbTableWrapper;
use rust_extensions::lazy::LazyVec;

use crate::{
    app::AppContext,
    db_operations::{read::read_operation_result::UpdateStatistics, DbOperationError},
};

use super::super::ReadOperationResult;

pub async fn get_single_partition_multiple_rows(
    app: &std::sync::Arc<AppContext>,
    db_table_wrapper: &DbTableWrapper,
    partition_key: &String,
    row_keys: Vec<String>,
    update_statistics: UpdateStatistics,
) -> Result<ReadOperationResult, DbOperationError> {
    super::super::super::check_app_states(app)?;

    let mut write_access = db_table_wrapper.data.write().await;

    let db_partition = write_access.get_partition_mut(partition_key);

    if db_partition.is_none() {
        return Ok(ReadOperationResult::EmptyArray);
    }

    let db_partition = db_partition.unwrap();

    let mut db_rows = LazyVec::with_capacity(row_keys.len());

    for row_key in &row_keys {
        let db_row = db_partition.get_row(row_key);

        if let Some(db_row) = db_row {
            db_rows.add(db_row);
        }
    }

    return Ok(ReadOperationResult::compile_array_or_empty_from_partition(
        app,
        &db_table_wrapper.name,
        partition_key,
        db_rows.get_result(),
        update_statistics,
    )
    .await);
}
