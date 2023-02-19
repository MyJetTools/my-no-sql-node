use my_no_sql_server_core::DbTableWrapper;
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext,
    db_operations::{read::read_operation_result::UpdateStatistics, DbOperationError},
};

use super::super::ReadOperationResult;

pub async fn get_all_by_partition_key(
    app: &std::sync::Arc<AppContext>,
    db_table_wrapper: &DbTableWrapper,
    partition_key: &String,
    limit: Option<usize>,
    skip: Option<usize>,
    update_statistics: UpdateStatistics,
) -> Result<ReadOperationResult, DbOperationError> {
    super::super::super::check_app_states(app)?;

    let now = DateTimeAsMicroseconds::now();

    let mut db_table = db_table_wrapper.data.write().await;

    db_table.last_read_time.update(now);

    let get_partition_result = db_table.get_partition_mut(partition_key);

    if get_partition_result.is_none() {
        return Ok(ReadOperationResult::EmptyArray);
    }

    let db_rows = crate::db_operations::read::read_filter::filter_it(
        get_partition_result.unwrap().get_all_rows(),
        limit,
        skip,
    );

    return Ok(ReadOperationResult::compile_array_or_empty_from_partition(
        app,
        &db_table_wrapper.name,
        partition_key,
        db_rows,
        update_statistics,
    )
    .await);
}
