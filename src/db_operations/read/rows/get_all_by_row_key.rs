use std::collections::HashMap;

use my_no_sql_server_core::DbTableWrapper;
use rust_extensions::lazy::LazyVec;

use crate::{
    app::AppContext,
    db_operations::{read::read_operation_result::UpdateStatistics, DbOperationError},
};

use super::super::ReadOperationResult;

pub async fn get_all_by_row_key(
    app: &std::sync::Arc<AppContext>,
    db_table_wrapper: &DbTableWrapper,
    row_key: &str,
    limit: Option<usize>,
    skip: Option<usize>,
    update_statistics: UpdateStatistics,
) -> Result<ReadOperationResult, DbOperationError> {
    super::super::super::check_app_states(app)?;

    let mut db_table = db_table_wrapper.data.write().await;

    let mut db_rows = LazyVec::new();

    for partition in db_table.partitions.get_partitions_mut() {
        let get_row_result = partition.get_row(row_key);

        if let Some(db_row) = get_row_result {
            db_rows.add(db_row);
        }
    }

    let db_rows = db_rows.get_result();

    if db_rows.is_none() {
        return Ok(ReadOperationResult::EmptyArray);
    }

    let db_rows = crate::db_operations::read::read_filter::filter_it(
        db_rows.unwrap().into_iter(),
        limit,
        skip,
    );

    let db_rows = if let Some(db_rows) = db_rows {
        let mut result = HashMap::new();
        for db_row in db_rows {
            result.insert(db_row.partition_key.to_string(), vec![db_row]);
        }

        Some(result)
    } else {
        None
    };

    return Ok(ReadOperationResult::compile_array_or_empty(
        app,
        db_table_wrapper.name.as_str(),
        db_rows,
        update_statistics,
    )
    .await);
}
