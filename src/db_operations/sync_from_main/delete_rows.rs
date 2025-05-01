use std::sync::Arc;

use my_no_sql_sdk::{core::db::PartitionKey, tcp_contracts::DeleteRowTcpContract};

use crate::{
    app::AppContext,
    db_sync::{states::DeleteRowsEventSyncData, SyncEvent},
};

pub async fn delete_rows(
    app: &Arc<AppContext>,
    table_name: String,
    rows: Vec<DeleteRowTcpContract>,
) {
    let db_table = super::get_or_add_table(app, table_name.as_str()).await;

    let mut table_data = db_table.data.write().await;

    let mut sync_data = DeleteRowsEventSyncData::new(&table_data);

    for db_row in rows {
        let partition_key = PartitionKey::new(db_row.partition_key);

        let removed_row = table_data.remove_row(&partition_key, &db_row.row_key, true);

        if let Some(deleted_row) = removed_row {
            let (partition_key, deleted_row, _) = deleted_row;
            sync_data.add_deleted_row(partition_key.as_str(), deleted_row);
        }
    }

    app.dispatch(SyncEvent::DeleteRows(sync_data));
}
