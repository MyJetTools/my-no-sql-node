use my_no_sql_tcp_shared::{DeleteRowTcpContract, TcpContract};

use crate::db_sync::SyncEvent;
use my_json::json_reader::consts::EMPTY_ARRAY;

pub async fn serialize(sync_event: &SyncEvent) -> Option<Vec<u8>> {
    match sync_event {
        SyncEvent::TableFirstInit(sync_data) => {
            let table_snapshot = sync_data.db_table.get_table_snapshot().await;

            let data = table_snapshot.as_json_array().build();

            let tcp_contract = TcpContract::InitTable {
                table_name: sync_data.db_table.name.to_string(),
                data,
            };

            tcp_contract.serialize().into()
        }
        SyncEvent::UpdateTableAttributes(_) => None,
        SyncEvent::InitTable(sync_data) => {
            let data = sync_data.table_snapshot.as_json_array().build();

            let result = TcpContract::InitTable {
                table_name: sync_data.table_data.table_name.to_string(),
                data,
            };

            result.serialize().into()
        }
        SyncEvent::InitPartitions(data) => {
            let mut result = Vec::new();

            for (partition_key, snapshot) in &data.partitions_to_update {
                let contract = TcpContract::InitPartition {
                    partition_key: partition_key.to_string(),
                    table_name: data.table_data.table_name.to_string(),
                    data: if let Some(db_partition_snapshot) = snapshot {
                        db_partition_snapshot
                            .db_rows_snapshot
                            .as_json_array()
                            .build()
                    } else {
                        EMPTY_ARRAY.to_vec()
                    },
                };

                contract.serialize_into(&mut result);
            }

            result.into()
        }
        SyncEvent::UpdateRows(data) => {
            let result = TcpContract::UpdateRows {
                table_name: data.table_data.table_name.to_string(),
                data: data.rows_by_partition.as_json_array().build(),
            };
            result.serialize().into()
        }
        SyncEvent::DeleteRows(data) => {
            let mut result = Vec::new();

            if let Some(deleted_partitions) = &data.deleted_partitions {
                for (partition_key, _) in deleted_partitions {
                    TcpContract::InitPartition {
                        table_name: data.table_data.table_name.to_string(),
                        partition_key: partition_key.to_string(),
                        data: EMPTY_ARRAY.to_vec(),
                    }
                    .serialize_into(&mut result);
                }
            }

            if let Some(deleted_rows) = &data.deleted_rows {
                for (partition_key, rows) in deleted_rows {
                    let mut deleted_rows = Vec::new();

                    for row_key in rows.keys() {
                        let contract = DeleteRowTcpContract {
                            partition_key: partition_key.to_string(),
                            row_key: row_key.to_string(),
                        };

                        deleted_rows.push(contract);
                    }

                    let contract = TcpContract::DeleteRows {
                        table_name: data.table_data.table_name.to_string(),
                        rows: deleted_rows,
                    };

                    contract.serialize_into(&mut result);
                }
            }

            result.into()
        }
        SyncEvent::DeleteTable(data) => {
            let contract = TcpContract::InitTable {
                table_name: data.table_data.table_name.to_string(),
                data: EMPTY_ARRAY.to_vec(),
            };

            contract.serialize().into()
        }
    }
}
