use my_http_utils::macros::{MyHttpInput, MyHttpObjectStructure};
use serde::{Deserialize, Serialize};

/// `GET /api/Partitions/Details`.
#[derive(MyHttpInput)]
pub struct GetPartitionsDetailsContract {
    #[http_header(name = "ns"; description = "Namespace to work in. Empty or absent means the default namespace")]
    pub namespace: Option<String>,

    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,

    #[http_query(name = "filter"; description = "Only the partitions whose key contains this text, case-insensitive")]
    pub filter: Option<String>,

    #[http_query(name = "limit"; description = "How many partitions to return at most. The node never returns more than 1000")]
    pub limit: Option<usize>,
}

/// The partitions of a table matching the filter, with their metrics, in the table's partition
/// order - a page of them: a table can have hundreds of thousands.
#[derive(Serialize, Deserialize, MyHttpObjectStructure, Clone, Debug, PartialEq)]
pub struct PartitionsDetailsContract {
    pub total: u64,
    pub matched: u64,
    pub partitions: Vec<PartitionDetailsContract>,
}

#[derive(Serialize, Deserialize, MyHttpObjectStructure, Clone, Debug, PartialEq)]
pub struct PartitionDetailsContract {
    #[serde(rename = "partitionKey")]
    pub partition_key: String,
    #[serde(rename = "recordsCount")]
    pub records_count: u64,
    #[serde(rename = "dataSize")]
    pub data_size: u64,
}
