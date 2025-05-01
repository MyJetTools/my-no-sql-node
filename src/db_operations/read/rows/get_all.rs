use std::collections::HashMap;

use my_json::json_writer::JsonArrayWriter;
use my_no_sql_sdk::{server::DbTable, tcp_contracts::sync_to_main::UpdateEntityStatisticsData};

use crate::{app::AppContext, db_operations::DbOperationError};

use super::super::ReadOperationResult;

pub async fn get_all(
    app: &std::sync::Arc<AppContext>,
    db_table: &DbTable,
    limit: Option<usize>,
    skip: Option<usize>,
    update_statistics: UpdateEntityStatisticsData,
) -> Result<ReadOperationResult, DbOperationError> {
    super::super::super::check_app_states(app)?;

    let table_inner = db_table.data.read().await;

    let mut db_rows_statistics: HashMap<
        String,
        Vec<std::sync::Arc<my_no_sql_sdk::core::db::DbRow>>,
    > = HashMap::new();
    let has_update_statistics = update_statistics.has_data_to_update();
    let mut json_array_writer = JsonArrayWriter::new();
    for (db_partition, db_row) in table_inner.get_all_rows(skip, limit) {
        //update_statistics.update(db_table_wrapper, db_partition, Some(db_row), now);
        if has_update_statistics {
            match db_rows_statistics.get_mut(db_partition.partition_key.as_str()) {
                Some(db_rows_by_partition) => {
                    db_rows_by_partition.push(db_row.clone());
                }
                None => {
                    db_rows_statistics
                        .insert(db_partition.partition_key.to_string(), vec![db_row.clone()]);
                }
            }
        }
        json_array_writer.write(db_row.as_ref());
    }

    drop(table_inner);

    for (partition_key, db_rows) in db_rows_statistics {
        app.sync_to_main_node
            .update(
                db_table.name.as_str(),
                &partition_key,
                || db_rows.iter().map(|itm| itm.get_row_key()),
                &update_statistics,
            )
            .await;
    }

    return Ok(ReadOperationResult::RowsArray(
        json_array_writer.build().into_bytes(),
    ));
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
