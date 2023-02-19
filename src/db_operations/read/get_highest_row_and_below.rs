use std::sync::Arc;

use my_no_sql_server_core::DbTableWrapper;

use crate::{app::AppContext, db_operations::DbOperationError};

use super::{read_operation_result::UpdateStatistics, ReadOperationResult};

pub async fn get_highest_row_and_below(
    app: &Arc<AppContext>,
    db_table_wrapper: &DbTableWrapper,
    partition_key: &String,
    row_key: &String,
    limit: Option<usize>,
    update_statistics: UpdateStatistics,
) -> Result<ReadOperationResult, DbOperationError> {
    super::super::check_app_states(app)?;

    let read_access = db_table_wrapper.data.read().await;

    let db_partition = read_access.get_partition(partition_key);

    if db_partition.is_none() {
        return Ok(ReadOperationResult::EmptyArray);
    }

    let db_partition = db_partition.unwrap();

    let db_rows = db_partition.get_highest_row_and_below(row_key, limit);

    return Ok(ReadOperationResult::compile_array_or_empty_from_partition(
        app,
        db_table_wrapper.name.as_str(),
        partition_key,
        db_rows,
        update_statistics,
    )
    .await);
}
