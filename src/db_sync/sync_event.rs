use super::states::{
    DeleteRowsEventSyncData, DeleteTableSyncData, InitPartitionsSyncData, InitTableEventSyncData,
    TableFirstInitSyncData, UpdateRowsSyncData, UpdateTableAttributesSyncData,
};

pub enum SyncEvent {
    UpdateTableAttributes(UpdateTableAttributesSyncData),

    InitTable(InitTableEventSyncData),

    InitPartitions(InitPartitionsSyncData),

    UpdateRows(UpdateRowsSyncData),

    DeleteRows(DeleteRowsEventSyncData),

    DeleteTable(DeleteTableSyncData),

    TableFirstInit(TableFirstInitSyncData),
}

impl SyncEvent {
    pub fn get_table_name(&self) -> &str {
        match self {
            SyncEvent::UpdateTableAttributes(data) => data.table_data.table_name.as_ref(),
            SyncEvent::InitTable(data) => data.table_data.table_name.as_ref(),
            SyncEvent::InitPartitions(data) => data.table_data.table_name.as_ref(),
            SyncEvent::UpdateRows(data) => data.table_data.table_name.as_ref(),
            SyncEvent::DeleteRows(data) => data.table_data.table_name.as_ref(),
            SyncEvent::DeleteTable(data) => data.table_data.table_name.as_ref(),
            SyncEvent::TableFirstInit(data) => data.db_table.name.as_ref(),
        }
    }
}

impl Into<SyncEvent> for InitTableEventSyncData {
    fn into(self) -> SyncEvent {
        SyncEvent::InitTable(self)
    }
}
