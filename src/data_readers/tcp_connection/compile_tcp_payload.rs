use my_no_sql_sdk::tcp_contracts::{DeleteRowTcpContract, MyNoSqlTcpContract};

use crate::db_sync::SyncEvent;
use my_json::consts::EMPTY_ARRAY;

/// Serializes a change into the packets a TCP reader receives.
pub fn compile_tcp_payload(sync_event: &SyncEvent) -> Vec<MyNoSqlTcpContract> {
    match sync_event {
        SyncEvent::TableFirstInit(sync_data) => {
            vec![MyNoSqlTcpContract::InitTable {
                table_name: sync_data.db_table.name.to_string(),
                data: sync_data
                    .db_table
                    .get_table_as_json_array()
                    .build()
                    .into_bytes(),
            }]
        }

        SyncEvent::InitTable(sync_data) => {
            vec![MyNoSqlTcpContract::InitTable {
                table_name: sync_data.db_table.name.to_string(),
                data: sync_data
                    .db_table
                    .get_table_as_json_array()
                    .build()
                    .into_bytes(),
            }]
        }

        SyncEvent::InitPartitions(sync_data) => sync_data
            .partitions_to_update
            .iter()
            .map(
                |(partition_key, snapshot)| MyNoSqlTcpContract::InitPartition {
                    partition_key: partition_key.to_string(),
                    table_name: sync_data.table_name.to_string(),
                    data: match snapshot {
                        Some(db_partition_snapshot) => db_partition_snapshot
                            .db_rows_snapshot
                            .as_json_array()
                            .build()
                            .into_bytes(),
                        None => EMPTY_ARRAY.to_vec(),
                    },
                },
            )
            .collect(),

        SyncEvent::UpdateRows(sync_data) => {
            vec![MyNoSqlTcpContract::UpdateRows {
                table_name: sync_data.table_name.to_string(),
                data: sync_data
                    .rows_by_partition
                    .as_json_array()
                    .build()
                    .into_bytes(),
            }]
        }

        SyncEvent::DeleteRows(sync_data) => sync_data
            .deleted_rows
            .iter()
            .map(|(partition_key, rows)| MyNoSqlTcpContract::DeleteRows {
                table_name: sync_data.table_name.to_string(),
                rows: rows
                    .keys()
                    .map(|row_key| DeleteRowTcpContract {
                        partition_key: partition_key.to_string(),
                        row_key: row_key.to_string(),
                    })
                    .collect(),
            })
            .collect(),
    }
}
