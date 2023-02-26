use std::sync::Arc;

use my_no_sql_core::db_json_entity::DbJsonEntity;

use crate::{
    app::AppContext,
    db_sync::{states::InitPartitionsSyncData, EventSource, SyncEvent},
};

pub async fn sync_partition(
    app: &Arc<AppContext>,
    table_name: String,
    partition_key: String,
    data: Vec<u8>,
    event_src: EventSource,
) {
    let db_table = super::get_or_add_table(app, table_name.as_str()).await;

    let entities = DbJsonEntity::restore_as_vec(data.as_slice()).unwrap();

    let mut table_data = db_table.data.write().await;

    table_data.remove_partition(&partition_key);

    table_data.bulk_insert_or_replace(&partition_key, &entities);

    let sync_data = InitPartitionsSyncData::new_as_update_partition(
        &table_data,
        partition_key.as_str(),
        event_src,
    );
    crate::operations::sync::dispatch(app, SyncEvent::InitPartitions(sync_data));
}
