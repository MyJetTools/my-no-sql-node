use super::{
    DeleteRowsEventSyncData, InitPartitionsSyncData, InitTableEventSyncData,
    TableFirstInitSyncData, UpdateRowsSyncData,
};

pub enum SyncEvent {
    InitTable(InitTableEventSyncData),

    InitPartitions(InitPartitionsSyncData),

    UpdateRows(UpdateRowsSyncData),

    DeleteRows(DeleteRowsEventSyncData),

    /// The snapshot of a table for the one reader which has just subscribed to it.
    TableFirstInit(TableFirstInitSyncData),
}

impl SyncEvent {
    pub fn get_table_name(&self) -> &str {
        match self {
            SyncEvent::InitTable(data) => data.db_table.name.as_str(),
            SyncEvent::InitPartitions(data) => data.table_name.as_str(),
            SyncEvent::UpdateRows(data) => data.table_name.as_str(),
            SyncEvent::DeleteRows(data) => data.table_name.as_str(),
            SyncEvent::TableFirstInit(data) => data.db_table.name.as_str(),
        }
    }
}
