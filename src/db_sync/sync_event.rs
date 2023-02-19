use super::states::{
    DeleteRowsEventSyncData, InitPartitionsSyncData, InitTableEventSyncData,
    TableFirstInitSyncData, UpdateRowsSyncData,
};

pub enum SyncEvent {
    InitTable(InitTableEventSyncData),

    InitPartitions(InitPartitionsSyncData),

    UpdateRows(UpdateRowsSyncData),

    DeleteRows(DeleteRowsEventSyncData),

    TableFirstInit(TableFirstInitSyncData),
}

impl SyncEvent {
    pub fn get_table_name(&self) -> &str {
        match self {
            SyncEvent::InitTable(data) => data.db_table.name.as_ref(),
            SyncEvent::InitPartitions(data) => data.table_name.as_ref(),
            SyncEvent::UpdateRows(data) => data.table_name.as_ref(),
            SyncEvent::DeleteRows(data) => data.table_name.as_ref(),
            SyncEvent::TableFirstInit(data) => data.db_table.name.as_ref(),
        }
    }
}

impl Into<SyncEvent> for InitTableEventSyncData {
    fn into(self) -> SyncEvent {
        SyncEvent::InitTable(self)
    }
}
