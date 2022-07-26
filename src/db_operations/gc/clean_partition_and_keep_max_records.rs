use my_no_sql_core::db::DbTable;
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext,
    db_operations::DbOperationError,
    db_sync::{states::DeleteRowsEventSyncData, EventSource, SyncEvent},
};

pub async fn execute(
    app: &AppContext,
    db_table: &DbTable,
    partition_key: &str,
    max_rows_amount: usize,
    event_source: EventSource,
    sync_moment: DateTimeAsMicroseconds,
) -> Result<(), DbOperationError> {
    super::super::check_app_states(app)?;

    let mut table_data = db_table.data.write().await;

    let partition = table_data.get_partition_mut(partition_key);

    if partition.is_none() {
        return Ok(());
    }

    let partition = partition.unwrap();

    let gced_rows_result = partition.gc_rows(max_rows_amount);

    if let Some(gced_rows) = gced_rows_result {
        app.persist_markers
            .persist_partition(&table_data.name.as_str(), partition_key, sync_moment)
            .await;

        let mut sync_data = DeleteRowsEventSyncData::new(
            &table_data,
            db_table.attributes.get_persist(),
            event_source,
        );

        sync_data.add_deleted_rows(partition_key, &gced_rows);
        crate::operations::sync::dispatch(app, SyncEvent::DeleteRows(sync_data));
    }

    Ok(())
}
