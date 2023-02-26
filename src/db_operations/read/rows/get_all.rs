use std::collections::HashMap;

use my_no_sql_server_core::DbTableWrapper;

use crate::{
    app::AppContext,
    db_operations::{read::read_operation_result::UpdateStatistics, DbOperationError},
};

use super::super::ReadOperationResult;

pub async fn get_all(
    app: &std::sync::Arc<AppContext>,
    db_table_wrapper: &DbTableWrapper,
    limit: Option<usize>,
    skip: Option<usize>,
    update_statistics: UpdateStatistics,
) -> Result<ReadOperationResult, DbOperationError> {
    super::super::super::check_app_states(app)?;

    let table_data = db_table_wrapper.data.read().await;

    let result_items = table_data.get_all_rows();

    let db_rows =
        crate::db_operations::read::read_filter::filter_it(result_items.into_iter(), limit, skip);

    let db_rows = if let Some(db_rows) = db_rows {
        let mut result = HashMap::new();

        for db_row in db_rows {
            if !result.contains_key(&db_row.partition_key) {
                result.insert(db_row.partition_key.clone(), Vec::new());
            }
            result.get_mut(&db_row.partition_key).unwrap().push(db_row);
        }

        Some(result)
    } else {
        None
    };

    return Ok(ReadOperationResult::compile_array_or_empty(
        app,
        &db_table_wrapper.name,
        db_rows,
        update_statistics,
    )
    .await);
}

/*
async fn get_all_and_no_expiration_time_update(
    db_table_wrapper: &DbTableWrapper,
    limit: Option<usize>,
    skip: Option<usize>,
) -> ReadOperationResult {
    let now = DateTimeAsMicroseconds::now();

    let table_data = db_table_wrapper.data.read().await;

    table_data.last_read_time.update(now);

    let mut json_array_writer = JsonArrayWriter::new();

    let db_rows = DbRowsFilter::new(table_data.get_all_rows().into_iter(), limit, skip);

    for db_row in  {
        json_array_writer.write_raw_element(&db_row.data);
        db_row.last_read_access.update(now);
    }

    Ok(ReadOperationResult::RowsArray(json_array_writer.build()))
}

async fn get_all_and_update_expiration_time(
    db_table_wrapper: &DbTableWrapper,
    limit: Option<usize>,
    skip: Option<usize>,
    update_expiration_time: &UpdateExpirationTimeModel,
) -> ReadOperationResult {

}
 */
