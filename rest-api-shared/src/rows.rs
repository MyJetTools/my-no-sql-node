use my_http_utils::macros::MyHttpInput;

/// `GET /api/Row` - a single row when both keys are given, otherwise the rows of the partition,
/// of the row key across the partitions, or of the whole table.
#[derive(MyHttpInput)]
pub struct GetRowInputModel {
    #[http_header(name = "ns"; description = "Namespace to work in. Empty or absent means the default namespace")]
    pub namespace: Option<String>,

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

    #[http_header(name = "updatePartitionLastReadTime"; description = "Update partition last read time")]
    pub update_partition_last_read_access_time: Option<bool>,

    #[http_header(name = "setPartitionExpirationTime"; description = "Set Partition Expiration time")]
    pub set_partition_expiration_time: Option<String>,

    #[http_header(name = "updateRowsLastReadTime"; description = "Update partition last read time")]
    pub update_db_rows_last_read_access_time: Option<bool>,

    #[http_header(name = "setRowsExpirationTime"; description = "Set Found DbRows Expiration time")]
    pub set_db_rows_expiration_time: Option<String>,
}
