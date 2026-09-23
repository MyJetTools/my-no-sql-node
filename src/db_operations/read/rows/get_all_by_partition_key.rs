use my_no_sql_sdk::{server::DbTable, tcp_contracts::sync_to_main::UpdateEntityStatisticsData};

use crate::{db_operations::read::ReadOperationResult, namespaces::NodeNamespace};

pub fn get_all_by_partition_key(
    namespace: &NodeNamespace,
    db_table: &DbTable,
    partition_key: &str,
    limit: Option<usize>,
    skip: Option<usize>,
    update_statistics: UpdateEntityStatisticsData,
) -> ReadOperationResult {
    let (json_array_writer, db_rows) = {
        let table_inner = db_table.data.read();

        let Some(db_partition) = table_inner.get_partition(partition_key) else {
            return ReadOperationResult::EmptyArray;
        };

        crate::db_operations::read::filter_and_compile_json(
            db_partition.get_all_rows(),
            limit,
            skip,
        )
    };

    if update_statistics.has_data_to_update() && !db_rows.is_empty() {
        namespace.main_node.sync_to_main_node.update(
            db_table.name.as_str(),
            partition_key,
            || db_rows.iter().map(|itm| itm.get_row_key()),
            &update_statistics,
        );
    }

    ReadOperationResult::RowsArray(json_array_writer.build().into_bytes())
}
