use std::sync::Arc;

use my_no_sql_server_core::DbTableWrapper;

use crate::data_readers::DataReader;

pub struct TableFirstInitSyncData {
    pub db_table: Arc<DbTableWrapper>,
    pub data_reader: Arc<DataReader>,
}
