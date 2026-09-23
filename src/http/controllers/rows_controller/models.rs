use my_http_server::macros::*;
use my_http_server::RawDataTyped;
use my_no_sql_sdk::tcp_contracts::sync_to_main::UpdateEntityStatisticsData;

use crate::http::controllers::UpdateStatisticsHeaders;

#[derive(MyHttpInput)]
pub struct GetHighestRowsAndBelowInputContract {
    #[http_header(name = "ns"; description = "Namespace to work in. Empty or absent means the default namespace")]
    pub namespace: Option<String>,

    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,

    #[http_query(name = "partitionKey"; description = "Partition Key")]
    pub partition_key: String,

    #[http_query(name = "rowKey"; description = "Highest row key")]
    pub row_key: String,

    #[http_query(name = "maxAmount"; description = "Limit amount of records we are going to get")]
    pub max_amount: Option<usize>,

    #[http_header(name = "updatePartitionLastReadTime"; description = "Update partition last read time")]
    pub update_partition_last_read_access_time: Option<bool>,

    #[http_header(name = "setPartitionExpirationTime"; description = "Set Partition Expiration time")]
    pub set_partition_expiration_time: Option<String>,

    #[http_header(name = "updateRowsLastReadTime"; description = "Update partition last read time")]
    pub update_db_rows_last_read_access_time: Option<bool>,

    #[http_header(name = "setRowsExpirationTime"; description = "Set Found DbRows Expiration time")]
    pub set_db_rows_expiration_time: Option<String>,
}

impl GetHighestRowsAndBelowInputContract {
    pub fn get_update_statistics(&self) -> UpdateEntityStatisticsData {
        UpdateStatisticsHeaders {
            update_partition_last_read_time: self.update_partition_last_read_access_time,
            set_partition_expiration_time: self.set_partition_expiration_time.as_deref(),
            update_rows_last_read_time: self.update_db_rows_last_read_access_time,
            set_rows_expiration_time: self.set_db_rows_expiration_time.as_deref(),
        }
        .into()
    }
}

#[derive(MyHttpInput)]
pub struct GetSinglePartitionMultipleRowsActionInputContract {
    #[http_header(name = "ns"; description = "Namespace to work in. Empty or absent means the default namespace")]
    pub namespace: Option<String>,

    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,

    #[http_query(name = "partitionKey"; description = "Partition Key")]
    pub partition_key: String,

    #[http_body_raw(description = "Row keys")]
    pub body: RawDataTyped<Vec<String>>,

    #[http_header(name = "updatePartitionLastReadTime"; description = "Update partition last read time")]
    pub update_partition_last_read_access_time: Option<bool>,

    #[http_header(name = "setPartitionExpirationTime"; description = "Set Partition Expiration time")]
    pub set_partition_expiration_time: Option<String>,

    #[http_header(name = "updateRowsLastReadTime"; description = "Update partition last read time")]
    pub update_db_rows_last_read_access_time: Option<bool>,

    #[http_header(name = "setRowsExpirationTime"; description = "Set Found DbRows Expiration time")]
    pub set_db_rows_expiration_time: Option<String>,
}

impl GetSinglePartitionMultipleRowsActionInputContract {
    pub fn get_update_statistics(&self) -> UpdateEntityStatisticsData {
        UpdateStatisticsHeaders {
            update_partition_last_read_time: self.update_partition_last_read_access_time,
            set_partition_expiration_time: self.set_partition_expiration_time.as_deref(),
            update_rows_last_read_time: self.update_db_rows_last_read_access_time,
            set_rows_expiration_time: self.set_db_rows_expiration_time.as_deref(),
        }
        .into()
    }
}
