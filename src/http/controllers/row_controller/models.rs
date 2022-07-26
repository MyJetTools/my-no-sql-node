use my_http_server_swagger::*;
use serde::{Deserialize, Serialize};

#[derive(MyHttpInput)]
pub struct RowsCountInputContract {
    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,

    #[http_query(name = "partitionKey"; description = "Partition Key")]
    pub partition_key: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct BaseDbRowContract {
    #[serde(rename = "partitionKey")]
    pub partition_key: String,

    #[serde(rename = "rowKey")]
    pub row_key: String,

    #[serde(rename = "timeStamp")]
    pub time_stamp: String,

    pub expires: Option<String>,
}

#[derive(MyHttpInput)]
pub struct GetRowInputModel {
    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,

    #[http_query(name = "partitionKey"; description = "Partition Key")]
    pub partition_key: Option<String>,

    #[http_query(name = "rowKey"; description = "Row Key")]
    pub row_key: Option<String>,

    #[http_query(name = "limit"; description = "Limit amount of records we are going to get")]
    pub limit: Option<usize>,

    #[http_query(name = "skip"; description = "Skip amount of records before start collecting them")]
    pub skip: Option<usize>,

    #[http_header(name ="setPartitionExpirationTime" description = "Set Partition Expiration time")]
    pub set_partition_expiration_time: Option<String>,

    #[http_header(name ="setRowsExpirationTime" description = "Set Found DbRows Expiration time")]
    pub set_db_rows_expiration_time: Option<String>,
}
