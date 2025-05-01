use my_http_server::macros::*;
use my_http_server::types::RawDataTyped;
use my_no_sql_sdk::{
    core::rust_extensions::date_time::DateTimeAsMicroseconds,
    tcp_contracts::sync_to_main::UpdateEntityStatisticsData,
};
use serde::{Deserialize, Serialize};

#[derive(MyHttpInput)]
pub struct GetHighestRowsAndBelowInputContract {
    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,

    #[http_query(name = "partitionKey"; description = "Partition Key")]
    pub partition_key: String,

    #[http_query(name = "rowKey"; description = "Highest row key")]
    pub row_key: String,

    #[http_query(name = "maxAmount"; description = "Limit amount of records we are going to get")]
    pub max_amount: Option<usize>,

    #[http_header(name ="updatePartitionLastReadTime"; description = "Update partition last read time")]
    pub update_partition_last_read_access_time: Option<bool>,

    #[http_header(name ="setPartitionExpirationTime"; description = "Set Partition Expiration time")]
    pub set_partition_expiration_time: Option<String>,

    #[http_header(name ="updateRowsLastReadTime"; description = "Update partition last read time")]
    pub update_db_rows_last_read_access_time: Option<bool>,

    #[http_header(name ="setRowsExpirationTime" description = "Set Found DbRows Expiration time")]
    pub set_db_rows_expiration_time: Option<String>,
}

impl GetHighestRowsAndBelowInputContract {
    pub fn get_partition_expiration_time(&self) -> Option<Option<DateTimeAsMicroseconds>> {
        let result = self.set_partition_expiration_time.as_ref()?;
        Some(DateTimeAsMicroseconds::from_str(result))
    }

    pub fn get_row_expiration_time(&self) -> Option<Option<DateTimeAsMicroseconds>> {
        let result = self.set_db_rows_expiration_time.as_ref()?;
        Some(DateTimeAsMicroseconds::from_str(result))
    }

    pub fn get_update_statistics(&self) -> UpdateEntityStatisticsData {
        UpdateEntityStatisticsData {
            partition_last_read_moment: if let Some(value) =
                self.update_partition_last_read_access_time
            {
                value
            } else {
                false
            },
            row_last_read_moment: if let Some(value) = self.update_db_rows_last_read_access_time {
                value
            } else {
                false
            },
            partition_expiration_moment: self.get_partition_expiration_time(),
            row_expiration_moment: self.get_row_expiration_time(),
        }
    }
}

#[derive(MyHttpInput)]
pub struct GetSinglePartitionMultipleRowsActionInputContract {
    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,

    #[http_query(name = "partitionKey"; description = "Partition Key")]
    pub partition_key: String,

    #[http_body_raw(description = "Row keys")]
    pub body: RawDataTyped<Vec<String>>,

    #[http_header(name ="updatePartitionLastReadTime"; description = "Update partition last read time")]
    pub update_partition_last_read_access_time: Option<bool>,

    #[http_header(name ="setPartitionExpirationTime"; description = "Set Partition Expiration time")]
    pub set_partition_expiration_time: Option<String>,

    #[http_header(name ="updateRowsLastReadTime"; description = "Update partition last read time")]
    pub update_db_rows_last_read_access_time: Option<bool>,

    #[http_header(name ="setRowsExpirationTime" description = "Set Found DbRows Expiration time")]
    pub set_db_rows_expiration_time: Option<String>,
}

impl GetSinglePartitionMultipleRowsActionInputContract {
    pub fn get_partition_expiration_time(&self) -> Option<Option<DateTimeAsMicroseconds>> {
        let result = self.set_partition_expiration_time.as_ref()?;
        Some(DateTimeAsMicroseconds::from_str(result))
    }

    pub fn get_row_expiration_time(&self) -> Option<Option<DateTimeAsMicroseconds>> {
        let result = self.set_db_rows_expiration_time.as_ref()?;
        Some(DateTimeAsMicroseconds::from_str(result))
    }

    pub fn get_update_statistics(&self) -> UpdateEntityStatisticsData {
        UpdateEntityStatisticsData {
            partition_last_read_moment: if let Some(value) =
                self.update_partition_last_read_access_time
            {
                value
            } else {
                false
            },
            row_last_read_moment: if let Some(value) = self.update_db_rows_last_read_access_time {
                value
            } else {
                false
            },
            partition_expiration_moment: self.get_partition_expiration_time(),
            row_expiration_moment: self.get_row_expiration_time(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct DeletePartitionsModel {
    #[serde(rename = "partitionKeys")]
    pub partition_keys: Vec<String>,
}
