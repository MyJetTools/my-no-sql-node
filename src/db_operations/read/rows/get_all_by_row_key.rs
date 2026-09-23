use my_json::json_writer::JsonArrayWriter;
use my_no_sql_sdk::{server::DbTable, tcp_contracts::sync_to_main::UpdateEntityStatisticsData};

use crate::{db_operations::read::ReadOperationResult, namespaces::NodeNamespace};

pub fn get_all_by_row_key(
    namespace: &NodeNamespace,
    db_table: &DbTable,
    row_key: &str,
    limit: Option<usize>,
    skip: Option<usize>,
    update_statistics: UpdateEntityStatisticsData,
) -> ReadOperationResult {
    let has_statistics_to_update = update_statistics.has_data_to_update();

    let mut json_array_writer = JsonArrayWriter::new();
    let mut partition_keys = Vec::new();

    {
        let table_inner = db_table.data.read();

        for (db_partition, db_row) in table_inner.get_by_row_key(row_key, skip, limit) {
            if has_statistics_to_update {
                partition_keys.push(db_partition.partition_key.to_string());
            }

            json_array_writer = json_array_writer.write(db_row.as_ref());
        }
    }

    for partition_key in partition_keys {
        namespace.main_node.sync_to_main_node.update(
            db_table.name.as_str(),
            partition_key.as_str(),
            || [row_key].into_iter(),
            &update_statistics,
        );
    }

    ReadOperationResult::RowsArray(json_array_writer.build().into_bytes())
}
