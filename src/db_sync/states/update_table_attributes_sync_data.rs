use crate::db_sync::EventSource;

use super::SyncTableData;

pub struct UpdateTableAttributesSyncData {
    pub table_data: SyncTableData,
    pub event_src: EventSource,
    pub persist: bool,
    pub max_partitions_amount: Option<usize>,
}
