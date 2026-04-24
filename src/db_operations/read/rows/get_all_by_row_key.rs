use my_json::json_writer::JsonArrayWriter;
use my_no_sql_sdk::{server::DbTable, tcp_contracts::sync_to_main::UpdateEntityStatisticsData};

use crate::{app::AppContext, db_operations::DbOperationError};

use super::super::ReadOperationResult;

pub async fn get_all_by_row_key(
    app: &std::sync::Arc<AppContext>,
    db_table: &DbTable,
    row_key: &str,
    limit: Option<usize>,
    skip: Option<usize>,
    update_statistics: UpdateEntityStatisticsData,
) -> Result<ReadOperationResult, DbOperationError> {
    super::super::super::check_app_states(app)?;

    let table_data = db_table.data.read();

    let mut json_array_writer = JsonArrayWriter::new();
    for (db_partition, db_row) in table_data.get_by_row_key(row_key, skip, limit) {
        if update_statistics.has_data_to_update() {
            app.sync_to_main_node.update(
                db_table.name.as_str(),
                db_partition.partition_key.as_str(),
                || [row_key].into_iter(),
                &update_statistics,
            );
        }

        json_array_writer = json_array_writer.write(db_row.as_ref());
    }

    return Ok(ReadOperationResult::RowsArray(
        json_array_writer.build().into_bytes(),
    ));
}
