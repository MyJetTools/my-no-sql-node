use std::sync::Arc;

use my_no_sql_sdk::tcp_contracts::DeleteRowTcpContract;

use crate::{
    app::AppContext,
    db_sync::{states::DeleteRowsEventSyncData, EventSource, SyncEvent},
};

pub async fn delete_rows(
    app: &Arc<AppContext>,
    table_name: String,
    rows: Vec<DeleteRowTcpContract>,
    event_src: EventSource,
) {
    let db_table = super::get_or_add_table(app, table_name.as_str()).await;

    let mut table_data = db_table.data.write().await;

    let mut sync_data = DeleteRowsEventSyncData::new(&table_data, event_src);

    for db_row in rows {
        let removed_row =
            table_data.remove_row(&db_row.partition_key, db_row.row_key.as_str(), true);

        if let Some(deleted_row) = removed_row {
            sync_data.add_deleted_row(db_row.partition_key.as_str(), deleted_row.0);
        }
    }

    crate::operations::sync::dispatch(app, SyncEvent::DeleteRows(sync_data));
}
