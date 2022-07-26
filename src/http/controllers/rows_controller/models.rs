use my_http_server_swagger::*;
use serde::{Deserialize, Serialize};

#[derive(MyHttpInput)]
pub struct GetHighestRowsAndBelowInputContract {
    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,

    #[http_query(name = "partitionKey"; description = "Partition Key")]
    pub partition_key: String,

    #[http_query(name = "rowKey"; description = "Row Key")]
    pub row_key: String,

    #[http_query(name = "maxAmount"; description = "Limit amount of records we are going to get")]
    pub max_amount: Option<usize>,

    #[http_header(name ="setPartitionExpirationTime" description = "Set Partition Expiration time")]
    pub set_partition_expiration_time: Option<String>,

    #[http_header(name ="setRowsExpirationTime" description = "Set Found DbRows Expiration time")]
    pub set_db_rows_expiration_time: Option<String>,
}

#[derive(MyHttpInput)]
pub struct GetSinglePartitionMultipleRowsActionInputContract {
    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,

    #[http_query(name = "partitionKey"; description = "Partition Key")]
    pub partition_key: String,

    #[http_body(description = "Row keys")]
    pub body: Vec<u8>,

    #[http_header(name ="setPartitionExpirationTime" description = "Set Partition Expiration time")]
    pub set_partition_expiration_time: Option<String>,

    #[http_header(name ="setRowsExpirationTime" description = "Set Found DbRows Expiration time")]
    pub set_db_rows_expiration_time: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct DeletePartitionsModel {
    #[serde(rename = "partitionKeys")]
    pub partition_keys: Vec<String>,
}
