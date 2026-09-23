use std::{collections::BTreeMap, sync::Arc};

use my_json::json_writer::JsonArrayWriter;
use my_no_sql_sdk::{
    core::db::DbRow, server::DbTable, tcp_contracts::sync_to_main::UpdateEntityStatisticsData,
};

use crate::{db_operations::read::ReadOperationResult, namespaces::NodeNamespace};

pub fn get_all(
    namespace: &NodeNamespace,
    db_table: &DbTable,
    limit: Option<usize>,
    skip: Option<usize>,
    update_statistics: UpdateEntityStatisticsData,
) -> ReadOperationResult {
    let has_statistics_to_update = update_statistics.has_data_to_update();

    let mut db_rows_by_partition: BTreeMap<String, Vec<Arc<DbRow>>> = BTreeMap::new();
    let mut json_array_writer = JsonArrayWriter::new();

    {
        let table_inner = db_table.data.read();

        for (db_partition, db_row) in table_inner.get_all_rows(skip, limit) {
            if has_statistics_to_update {
                db_rows_by_partition
                    .entry(db_partition.partition_key.to_string())
                    .or_default()
                    .push(db_row.clone());
            }

            json_array_writer = json_array_writer.write(db_row.as_ref());
        }
    }

    for (partition_key, db_rows) in db_rows_by_partition {
        namespace.main_node.sync_to_main_node.update(
            db_table.name.as_str(),
            partition_key.as_str(),
            || db_rows.iter().map(|itm| itm.get_row_key()),
            &update_statistics,
        );
    }

    ReadOperationResult::RowsArray(json_array_writer.build().into_bytes())
}
