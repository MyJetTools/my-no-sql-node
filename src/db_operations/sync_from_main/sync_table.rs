use std::sync::Arc;

use my_no_sql_core::db_json_entity::JsonTimeStamp;

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
    let db_table = super::get_or_add_table(app, table_name.as_str()).await;

    let now = JsonTimeStamp::now();

    let entities =
        crate::db_operations::parse_json_entity::as_btree_map(data.as_slice(), &now).unwrap();

    let mut table_data = db_table.data.write().await;

    table_data.clean_table();

    for (partition_key, db_rows) in entities {
        table_data.bulk_insert_or_replace(partition_key.as_str(), &db_rows, &now);
    }

    let sync_data =
        InitTableEventSyncData::new(&table_data, db_table.attributes.get_snapshot(), event_src);

    crate::operations::sync::dispatch(app, SyncEvent::InitTable(sync_data));
}
