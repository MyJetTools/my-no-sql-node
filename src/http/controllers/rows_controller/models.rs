use my_http_server_swagger::*;
use serde::{Deserialize, Serialize};

use crate::{
    db_operations::read::UpdateStatistics, http::controllers::mappers::ToSetExpirationTime,
};

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
    pub fn get_update_statistics(&self) -> UpdateStatistics {
        UpdateStatistics {
            update_partition_last_read_access_time: if let Some(value) =
                self.update_partition_last_read_access_time
            {
                value
            } else {
                false
            },
            update_rows_last_read_access_time: if let Some(value) =
                self.update_db_rows_last_read_access_time
            {
                value
            } else {
                false
            },
            update_partition_expiration_time: self
                .set_partition_expiration_time
                .to_set_expiration_time(),
            update_rows_expiration_time: self.set_db_rows_expiration_time.to_set_expiration_time(),
        }
    }
}

#[derive(MyHttpInput)]
pub struct GetSinglePartitionMultipleRowsActionInputContract {
    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,

    #[http_query(name = "partitionKey"; description = "Partition Key")]
    pub partition_key: String,

    #[http_body(description = "Row keys")]
    pub body: Vec<u8>,

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
    pub fn get_update_statistics(&self) -> UpdateStatistics {
        UpdateStatistics {
            update_partition_last_read_access_time: if let Some(value) =
                self.update_partition_last_read_access_time
            {
                value
            } else {
                false
            },
            update_rows_last_read_access_time: if let Some(value) =
                self.update_db_rows_last_read_access_time
            {
                value
            } else {
                false
            },
            update_partition_expiration_time: self
                .set_partition_expiration_time
                .to_set_expiration_time(),
            update_rows_expiration_time: self.set_db_rows_expiration_time.to_set_expiration_time(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct DeletePartitionsModel {
    #[serde(rename = "partitionKeys")]
    pub partition_keys: Vec<String>,
}
