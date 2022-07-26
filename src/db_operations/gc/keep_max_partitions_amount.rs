use std::sync::Arc;

use my_no_sql_core::db::{DbTable, DbTableInner};
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext,
    db_operations::DbOperationError,
    db_sync::{states::InitPartitionsSyncData, EventSource, SyncEvent},
};

//TODO - Use Method from TableData
pub async fn keep_max_partitions_amount(
    app: &AppContext,
    db_table: &Arc<DbTable>,
    max_partitions_amount: usize,
    event_src: EventSource,
    persist_moment: DateTimeAsMicroseconds,
) -> Result<(), DbOperationError> {
    super::super::check_app_states(app)?;

    let partitions_amount = db_table.get_partitions_amount().await;

    if partitions_amount <= max_partitions_amount {
        return Ok(());
    }

    let mut table_data = db_table.data.write().await;

    gc_partitions(
        app,
        db_table.as_ref(),
        &mut table_data,
        event_src,
        max_partitions_amount,
        persist_moment,
    )
    .await?;

    Ok(())
}

pub async fn gc_partitions(
    app: &AppContext,
    db_table: &DbTable,
    table_data: &mut DbTableInner,
    event_src: EventSource,
    max_partitions_amount: usize,
    persist_moment: DateTimeAsMicroseconds,
) -> Result<(), DbOperationError> {
    super::super::check_app_states(app)?;

    let mut sync_state =
        InitPartitionsSyncData::new(&table_data, event_src, db_table.attributes.get_persist());

    let gced_partitions_result =
        table_data.gc_and_keep_max_partitions_amount(max_partitions_amount);

    if let Some(gced_partitions) = gced_partitions_result {
        for (partition_key, _) in gced_partitions {
            app.persist_markers
                .persist_partition(
                    &table_data.name.as_str(),
                    partition_key.as_ref(),
                    persist_moment,
                )
                .await;

            sync_state.add(partition_key, None);
        }

        crate::operations::sync::dispatch(app, SyncEvent::InitPartitions(sync_state));
    }

    Ok(())
}
