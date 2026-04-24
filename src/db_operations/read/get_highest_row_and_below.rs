use std::sync::Arc;

use my_json::json_writer::JsonArrayWriter;
use my_no_sql_sdk::{server::DbTable, tcp_contracts::sync_to_main::UpdateEntityStatisticsData};

use crate::{app::AppContext, db_operations::DbOperationError};

use super::ReadOperationResult;

pub async fn get_highest_row_and_below(
    app: &Arc<AppContext>,
    db_table: &DbTable,
    partition_key: &String,
    row_key: &String,
    limit: Option<usize>,
    update_statistics: UpdateEntityStatisticsData,
) -> Result<ReadOperationResult, DbOperationError> {
    super::super::check_app_states(app)?;

    let table_inner = db_table.data.read();

    let db_partition = table_inner.get_partition(partition_key);

    if db_partition.is_none() {
        return Ok(ReadOperationResult::EmptyArray);
    }

    let db_partition = db_partition.unwrap();

    let mut json_array_writer = JsonArrayWriter::new();
    let mut count = 0;

    let has_statistics_to_update = update_statistics.has_data_to_update();

    let mut db_rows = Vec::new();
    for db_row in db_partition.get_highest_row_and_below(row_key) {
        if let Some(limit) = limit {
            if count >= limit {
                break;
            }
        }
        //update_statistics.update(db_table_wrapper, db_partition, Some(db_row), now);
        if has_statistics_to_update {
            db_rows.push(db_row.clone());
        }

        json_array_writer = json_array_writer.write(db_row.as_ref());

        count += 1;
    }

    drop(table_inner);

    if db_rows.len() > 0 {
        app.sync_to_main_node.update(
            db_table.name.as_str(),
            partition_key,
            || db_rows.iter().map(|itm| itm.get_row_key()),
            &update_statistics,
        );
    }

    return Ok(ReadOperationResult::RowsArray(
        json_array_writer.build().into_bytes(),
    ));
}
