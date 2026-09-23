use my_json::json_writer::JsonArrayWriter;
use my_no_sql_sdk::{server::DbTable, tcp_contracts::sync_to_main::UpdateEntityStatisticsData};

use crate::{db_operations::read::ReadOperationResult, namespaces::NodeNamespace};

pub fn get_single_partition_multiple_rows(
    namespace: &NodeNamespace,
    db_table: &DbTable,
    partition_key: &str,
    row_keys: &[String],
    update_statistics: UpdateEntityStatisticsData,
) -> ReadOperationResult {
    let has_statistics_to_update = update_statistics.has_data_to_update();

    let mut json_array_writer = JsonArrayWriter::new();
    let mut db_rows = Vec::new();

    {
        let table_inner = db_table.data.read();

        let Some(db_partition) = table_inner.get_partition(partition_key) else {
            return ReadOperationResult::EmptyArray;
        };

        for row_key in row_keys {
            if let Some(db_row) = db_partition.get_row(row_key) {
                if has_statistics_to_update {
                    db_rows.push(db_row.clone());
                }

                json_array_writer = json_array_writer.write(db_row.as_ref());
            }
        }
    }

    if !db_rows.is_empty() {
        namespace.main_node.sync_to_main_node.update(
            db_table.name.as_str(),
            partition_key,
            || db_rows.iter().map(|itm| itm.get_row_key()),
            &update_statistics,
        );
    }

    ReadOperationResult::RowsArray(json_array_writer.build().into_bytes())
}
