use my_no_sql_sdk::{server::DbTable, tcp_contracts::sync_to_main::UpdateEntityStatisticsData};

use crate::{app::AppContext, db_operations::DbOperationError};

use super::super::ReadOperationResult;

pub async fn get_all_by_partition_key(
    app: &std::sync::Arc<AppContext>,
    db_table: &DbTable,
    partition_key: &String,
    limit: Option<usize>,
    skip: Option<usize>,
    update_statistics: UpdateEntityStatisticsData,
) -> Result<ReadOperationResult, DbOperationError> {
    super::super::super::check_app_states(app)?;

    let inner = db_table.data.read();

    let db_partition = inner.get_partition(partition_key);

    if db_partition.is_none() {
        return Ok(ReadOperationResult::EmptyArray);
    }

    let db_partition = db_partition.unwrap();

    let (json_array_writer, db_rows) = super::super::read_filter::filter_and_compile_json(
        db_partition.get_all_rows().into_iter(),
        limit,
        skip,
    );

    if update_statistics.has_data_to_update() {
        if db_rows.len() > 0 {
            app.sync_to_main_node.update(
                db_table.name.as_str(),
                partition_key,
                || db_rows.iter().map(|itm| itm.get_row_key()),
                &update_statistics,
            );
        }
    }

    return Ok(ReadOperationResult::RowsArray(
        json_array_writer.build().into_bytes(),
    ));
}
