use my_no_sql_sdk::core::db_json_entity::DbJsonEntity;

use crate::{
    app::AppContext,
    db_sync::{InitPartitionsSyncData, SyncEvent},
    namespaces::NodeNamespace,
};

/// The whole partition. An empty one means the partition is gone.
pub fn sync_partition(
    app: &AppContext,
    namespace: &NodeNamespace,
    table_name: String,
    partition_key: String,
    data: Vec<u8>,
) {
    let db_rows = match DbJsonEntity::restore_as_vec(data.as_slice()) {
        Ok(db_rows) => db_rows,
        Err(err) => {
            super::report_broken_payload("InitPartition", namespace, table_name.as_str(), &err);
            return;
        }
    };

    let Some(db_table) = namespace.db.get_table(table_name.as_str()) else {
        return;
    };

    let sync_data = {
        let mut table_data = db_table.data.write();

        table_data.remove_partition(&partition_key);

        // Not for an empty one: inserting would leave an empty partition behind.
        if !db_rows.is_empty() {
            table_data.bulk_insert_or_replace(&partition_key, &db_rows);
        }

        InitPartitionsSyncData::new_as_update_partition(&table_data, partition_key.as_str())
    };

    app.dispatch(namespace.name.clone(), SyncEvent::InitPartitions(sync_data));
}
