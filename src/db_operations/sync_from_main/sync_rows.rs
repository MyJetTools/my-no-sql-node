use my_no_sql_sdk::core::{db::PartitionKey, db_json_entity::DbJsonEntity};

use crate::{
    app::AppContext,
    db_sync::{SyncEvent, UpdateRowsSyncData},
    namespaces::NodeNamespace,
};

pub fn sync_rows(app: &AppContext, namespace: &NodeNamespace, table_name: String, data: Vec<u8>) {
    let entities = match DbJsonEntity::restore_grouped_by_partition_key(data.as_slice()) {
        Ok(entities) => entities,
        Err(err) => {
            super::report_broken_payload("UpdateRows", namespace, table_name.as_str(), &err);
            return;
        }
    };

    let Some(db_table) = namespace.db.get_table(table_name.as_str()) else {
        return;
    };

    let mut sync_data = UpdateRowsSyncData::new(db_table.name.clone());

    {
        let mut table_data = db_table.data.write();

        for (partition_key, db_rows) in entities {
            table_data.bulk_insert_or_replace(&partition_key, &db_rows);

            sync_data
                .rows_by_partition
                .add_rows(PartitionKey::new(partition_key), db_rows);
        }
    }

    if sync_data.rows_by_partition.has_elements() {
        app.dispatch(namespace.name.clone(), SyncEvent::UpdateRows(sync_data));
    }
}
