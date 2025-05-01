use my_no_sql_sdk::core::db::DbTableName;

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
    pub fn get_table_name(&self) -> &DbTableName {
        match self {
            SyncEvent::InitTable(data) => &data.db_table.name,
            SyncEvent::InitPartitions(data) => &data.table_name,
            SyncEvent::UpdateRows(data) => &data.table_name,
            SyncEvent::DeleteRows(data) => &data.table_name,
            SyncEvent::TableFirstInit(data) => &data.db_table.name,
        }
    }
}

impl Into<SyncEvent> for InitTableEventSyncData {
    fn into(self) -> SyncEvent {
        SyncEvent::InitTable(self)
    }
}
