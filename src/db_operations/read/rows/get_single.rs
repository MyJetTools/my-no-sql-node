use my_no_sql_sdk::{server::DbTable, tcp_contracts::sync_to_main::UpdateEntityStatisticsData};

use crate::{app::AppContext, db_operations::DbOperationError};

use super::super::ReadOperationResult;

pub async fn get_single(
    app: &std::sync::Arc<AppContext>,
    db_table: &DbTable,
    partition_key: &String,
    row_key: &str,
    update_statistics: UpdateEntityStatisticsData,
) -> Result<ReadOperationResult, DbOperationError> {
    super::super::super::check_app_states(app)?;

    let table_inner = db_table.data.read();

    let db_partition = table_inner.get_partition(partition_key);

    if db_partition.is_none() {
        return Err(DbOperationError::RecordNotFound);
    }

    let db_partition = db_partition.unwrap();

    let db_row = db_partition.get_row(row_key);

    if db_row.is_none() {
        return Err(DbOperationError::RecordNotFound);
    }

    let db_row = db_row.unwrap().clone();

    drop(table_inner);

    app.sync_to_main_node.update(
        db_table.name.as_str(),
        partition_key,
        || [db_row.get_row_key()].into_iter(),
        &update_statistics,
    );

    return Ok(ReadOperationResult::SingleRow(db_row.to_vec()));
}
