use std::sync::Arc;

use crate::{
    app::AppContext,
    db_sync::{states::UpdateRowsSyncData, EventSource, SyncEvent},
};

pub async fn sync_rows(
    app: &Arc<AppContext>,
    table_name: String,
    data: Vec<u8>,
    event_src: EventSource,
) {
    let db_table = super::get_or_add_table(app, table_name.as_str()).await;

    let entities =
        crate::db_operations::parse_json_entity::restore_as_btree_map(data.as_slice()).unwrap();

    let mut table_data = db_table.data.write().await;

    let mut sync_data = UpdateRowsSyncData::new(&table_data, event_src);

    for (partition_key, db_rows) in entities {
        table_data.bulk_insert_or_replace(partition_key.as_str(), &db_rows);

        sync_data
            .rows_by_partition
            .add_rows(partition_key.as_str(), db_rows);
    }

    crate::operations::sync::dispatch(app, SyncEvent::UpdateRows(sync_data));
}
