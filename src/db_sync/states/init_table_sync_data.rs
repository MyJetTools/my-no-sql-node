use my_no_sql_core::db::{db_snapshots::DbTableSnapshot, DbTableAttributesSnapshot, DbTableInner};

use crate::db_sync::EventSource;

use super::SyncTableData;

pub struct InitTableEventSyncData {
    pub table_data: SyncTableData,
    pub event_src: EventSource,
    pub table_snapshot: DbTableSnapshot,
}

impl InitTableEventSyncData {
    pub fn new(
        table_data: &DbTableInner,
        table_attrs: DbTableAttributesSnapshot,
        event_src: EventSource,
    ) -> Self {
        Self {
            table_data: SyncTableData::new(table_data, table_attrs.persist),
            event_src,
            table_snapshot: DbTableSnapshot::new(
                table_data.get_last_update_time(),
                table_data,
                table_attrs,
            ),
        }
    }
}
