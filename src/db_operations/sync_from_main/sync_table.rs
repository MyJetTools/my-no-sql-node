use std::sync::Arc;

use crate::{
    app::AppContext,
    db_sync::{states::InitTableEventSyncData, EventSource, SyncEvent},
};

pub async fn sync_table(
    app: &Arc<AppContext>,
    table_name: String,
    data: Vec<u8>,
    event_src: EventSource,
) {
    let db_table_wrapper = super::get_or_add_table(app, table_name.as_str()).await;

    let entities = crate::db_operations::parse_json_entity::as_btree_map(data.as_slice()).unwrap();

    let mut db_table = db_table_wrapper.data.write().await;

    db_table.clean_table();

    for (partition_key, db_rows) in entities {
        db_table.bulk_insert_or_replace(partition_key.as_str(), &db_rows);
    }

    let sync_data = InitTableEventSyncData::new(db_table_wrapper.clone(), event_src);

    crate::operations::sync::dispatch(app, SyncEvent::InitTable(sync_data));
}
