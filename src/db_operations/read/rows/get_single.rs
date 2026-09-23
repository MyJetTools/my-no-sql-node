use my_no_sql_sdk::{server::DbTable, tcp_contracts::sync_to_main::UpdateEntityStatisticsData};

use crate::{
    db_operations::{read::ReadOperationResult, DbOperationError},
    namespaces::NodeNamespace,
};

pub fn get_single(
    namespace: &NodeNamespace,
    db_table: &DbTable,
    partition_key: &str,
    row_key: &str,
    update_statistics: UpdateEntityStatisticsData,
) -> Result<ReadOperationResult, DbOperationError> {
    let db_row = {
        let table_inner = db_table.data.read();

        table_inner
            .get_partition(partition_key)
            .and_then(|db_partition| db_partition.get_row_and_clone(row_key))
    };

    let Some(db_row) = db_row else {
        return Err(DbOperationError::RecordNotFound);
    };

    namespace.main_node.sync_to_main_node.update(
        db_table.name.as_str(),
        partition_key,
        || [db_row.get_row_key()].into_iter(),
        &update_statistics,
    );

    Ok(ReadOperationResult::SingleRow(db_row.to_vec()))
}
