//! The contracts of the main node's write api - copies of its own, so the node's swagger
//! describes the writes exactly as the main node does. A request is parsed with them only to be
//! validated the way the main node would; what goes on to the main node is the request as it
//! came, never these structs.

use std::collections::HashMap;

use my_http_server::macros::*;
use my_http_server::RawDataTyped;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, MyHttpStringEnum)]
pub enum DataSynchronizationPeriod {
    #[http_enum_case(id:0; value:"i"; description="Immediately Persist")]
    Immediately,
    #[http_enum_case(id:1; value:"1"; description="Persist during 1 sec")]
    Sec1,
    #[http_enum_case(id:5; value:"5";  description="Persist during 5 sec"; default)]
    Sec5,
    #[http_enum_case(id:15; value: "15"; description="Persist during 15 sec")]
    Sec15,
    #[http_enum_case(id:30; value: "30"; description="Persist during 30 sec")]
    Sec30,
    #[http_enum_case(id: 60; value: "60"; description="Persist during 1 minute")]
    Min1,
    #[http_enum_case(id:6; value: "a"; description="Sync as soon as CPU schedules task")]
    Asap,
}

impl DataSynchronizationPeriod {
    // `MyHttpInput` expands a bare `default` on an enum field into `Type::create_default()?`,
    // while the `MyHttpStringEnum` derive only emits `Default`.
    pub fn create_default() -> Result<Self, my_http_utils::http_input::HttpParseError> {
        Ok(Self::default())
    }
}

// ---- Row ----

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct BaseDbRowContract {
    #[serde(rename = "PartitionKey")]
    pub partition_key: String,

    #[serde(rename = "RowKey")]
    pub row_key: String,

    #[serde(rename = "TimeStamp")]
    pub time_stamp: String,

    #[serde(rename = "Expires")]
    pub expires: Option<String>,
}

#[derive(MyHttpInput)]
pub struct InsertOrReplaceInputContract {
    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,

    #[http_query(name = "syncPeriod"; description = "Synchronization period"; default)]
    pub sync_period: DataSynchronizationPeriod,

    #[http_body_raw(description = "DbEntity description")]
    pub body: RawDataTyped<BaseDbRowContract>,
}

#[derive(MyHttpInput)]
pub struct InsertOrReplaceIfNewInputContract {
    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,

    #[http_query(name = "syncPeriod"; description = "Synchronization period"; default)]
    pub sync_period: DataSynchronizationPeriod,

    #[http_body_raw(
        description = "DbEntity description. Must carry a TimeStamp - the row is written only when it is new or its TimeStamp is greater than the stored one"
    )]
    pub body: RawDataTyped<BaseDbRowContract>,
}

#[derive(MyHttpInput)]
pub struct InsertInputContract {
    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,

    #[http_query(name = "syncPeriod"; description = "Synchronization period"; default)]
    pub sync_period: DataSynchronizationPeriod,

    #[http_body_raw(description = "DbEntity description")]
    pub body: RawDataTyped<BaseDbRowContract>,
}

#[derive(MyHttpInput)]
pub struct ReplaceInputContract {
    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,

    #[http_query(name = "syncPeriod"; description = "Synchronization period"; default)]
    pub sync_period: DataSynchronizationPeriod,

    #[http_body_raw(description = "DbEntity description")]
    pub body: RawDataTyped<BaseDbRowContract>,
}

#[derive(MyHttpInput)]
pub struct DeleteRowIfInputModel {
    #[http_header(name = "ns"; description = "Namespace to work in. Empty or absent means the default namespace")]
    pub namespace: Option<String>,
    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,

    #[http_query(name = "partitionKey"; description = "Partition Key")]
    pub partition_key: String,

    #[http_query(name = "rowKey"; description = "Row Key")]
    pub row_key: String,

    #[http_query(
        name = "timeStamp";
        description = "TimeStamp the row was read at. The row is deleted only when this is still the TimeStamp stored in the table"
    )]
    pub time_stamp: String,

    #[http_query(name = "syncPeriod"; description = "Synchronization period"; default)]
    pub sync_period: DataSynchronizationPeriod,
}

#[derive(MyHttpInput)]
pub struct DeleteRowInputModel {
    #[http_header(name = "ns"; description = "Namespace to work in. Empty or absent means the default namespace")]
    pub namespace: Option<String>,
    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,

    #[http_query(name = "partitionKey"; description = "Partition Key")]
    pub partition_key: String,

    #[http_query(name = "rowKey"; description = "Row Key")]
    pub row_key: String,

    #[http_query(name = "syncPeriod"; description = "Synchronization period"; default)]
    pub sync_period: DataSynchronizationPeriod,
}

// ---- Rows ----

#[derive(MyHttpInput)]
pub struct DeletePartitionsInputContract {
    #[http_header(name = "ns"; description = "Namespace to work in. Empty or absent means the default namespace")]
    pub namespace: Option<String>,
    #[http_query(name: "tableName"; description: "Name of a table")]
    pub table_name: String,

    #[http_query(name: "partitionKeys"; description: "Partition Keys to delete" )]
    pub partition_keys: Vec<String>,

    #[http_query(name: "syncPeriod"; description: "Synchronization period"; default)]
    pub sync_period: DataSynchronizationPeriod,
}

// ---- Bulk ----

#[derive(MyHttpInput)]
pub struct BulkDeleteInputContract {
    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,

    #[http_query(name = "syncPeriod"; description = "Synchronization period"; default)]
    pub sync_period: DataSynchronizationPeriod,

    #[http_body_raw(
        description = "PartitionToDelete1:[RowToDelete1, RowToDelete2, RowToDelete3],[PartitionToDelete1]:[RowToDelete1, RowToDelete2, RowToDelete3]"
    )]
    pub body: RawDataTyped<HashMap<String, Vec<BaseDbRowContract>>>,
}

#[derive(MyHttpInput)]
pub struct BulkDeleteIfInputContract {
    #[http_header(name = "ns"; description = "Namespace to work in. Empty or absent means the default namespace")]
    pub namespace: Option<String>,

    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,

    #[http_query(name = "syncPeriod"; description = "Synchronization period"; default)]
    pub sync_period: DataSynchronizationPeriod,

    #[http_body_raw(
        description = "Rows to delete: [{\"PartitionKey\":\"pk\",\"RowKey\":\"rk\",\"TimeStamp\":\"2026-08-09T16:44:39.5404\"}]. A row is deleted only when the TimeStamp stored in the table is still the one sent here"
    )]
    pub body: RawDataTyped<Vec<DeleteIfRowContract>>,
}

/// One row of a conditional bulk delete: which row, and the version the client read it at.
#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct DeleteIfRowContract {
    #[serde(rename = "PartitionKey")]
    pub partition_key: String,

    #[serde(rename = "RowKey")]
    pub row_key: String,

    #[serde(rename = "TimeStamp")]
    pub time_stamp: String,
}

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct BulkDeleteIfResponseContract {
    pub deleted: usize,
    pub skipped: Vec<SkippedDeleteIfRowContract>,
}

/// `Reason`: `TimeStampMismatch` - the row is there but it is not the version the client read;
/// `NotFound` - there is no such row at all.
#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct SkippedDeleteIfRowContract {
    #[serde(rename = "PartitionKey")]
    pub partition_key: String,

    #[serde(rename = "RowKey")]
    pub row_key: String,

    #[serde(rename = "Reason")]
    pub reason: String,
}

#[derive(MyHttpInput)]
pub struct CleanAndBulkInsertInputContract {
    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,

    #[http_query(name = "partitionKey"; description = "Partition Key to clean before bulk insert operation";)]
    pub partition_key: Option<String>,

    #[http_query(name = "syncPeriod"; description = "Synchronization period"; default)]
    pub sync_period: DataSynchronizationPeriod,

    #[http_query(
        name = "useTimestamp";
        description = "If true - use each entity's own TimeStamp (every entity must carry a valid one, otherwise the request fails with 400). If absent or false - the server assigns its own clock as before"
    )]
    pub use_timestamp: Option<bool>,

    #[http_body_raw(description = "DbRows")]
    pub body: RawDataTyped<Vec<BaseDbRowContract>>,
}

#[derive(MyHttpInput)]
pub struct CleanAndBulkInsertByChunksInputContract {
    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,

    #[http_query(name = "partitionKey"; description = "Partition Key to clean before bulk insert operation. Taken into account only when a new process is started";)]
    pub partition_key: Option<String>,

    #[http_query(name = "processId"; description = "Id of the process. Omit it to start a new process - the id is returned in the response";)]
    pub process_id: Option<String>,

    #[http_query(
        name = "useTimestamp";
        description = "If true - use each entity's own TimeStamp (every entity of this chunk must carry a valid one, otherwise the chunk is rejected with 400). If absent or false - the server assigns its own clock as before"
    )]
    pub use_timestamp: Option<bool>,

    #[http_header(name = "session", description = "Writer session id")]
    pub session_id: Option<String>,

    #[http_body_raw(description = "DbRows")]
    pub body: RawDataTyped<Vec<BaseDbRowContract>>,
}

#[derive(MyHttpInput)]
pub struct InsertOrReplaceIfNewByChunksInputContract {
    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,

    #[http_query(name = "processId"; description = "Id of the process. Omit it to start a new process - the id is returned in the response";)]
    pub process_id: Option<String>,

    #[http_header(name = "session", description = "Writer session id")]
    pub session_id: Option<String>,

    #[http_body_raw(
        description = "DbRows. Each row must carry a TimeStamp - on commit a row is written only when it is new or its TimeStamp is greater than the stored one"
    )]
    pub body: RawDataTyped<Vec<BaseDbRowContract>>,
}

#[derive(MyHttpInput)]
pub struct BulkProcessInputContract {
    #[http_query(name = "processId"; description = "Id of the process")]
    pub process_id: String,

    #[http_query(name = "syncPeriod"; description = "Synchronization period"; default)]
    pub sync_period: DataSynchronizationPeriod,

    #[http_header(name = "session", description = "Writer session id")]
    pub session_id: Option<String>,
}

#[derive(MyHttpInput)]
pub struct CancelBulkProcessInputContract {
    #[http_query(name = "processId"; description = "Id of the process")]
    pub process_id: String,

    #[http_header(name = "session", description = "Writer session id")]
    pub session_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct BulkProcessResponse {
    #[serde(rename = "processId")]
    pub process_id: String,
}

#[derive(MyHttpInput)]
pub struct BulkInsertOrReplaceInputContract {
    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,

    #[http_query(name = "syncPeriod"; description = "Synchronization period"; default)]
    pub sync_period: DataSynchronizationPeriod,

    #[http_query(
        name = "useTimestamp";
        description = "If true - use each entity's own TimeStamp (every entity must carry a valid one, otherwise the request fails with 400). If absent or false - the server assigns its own clock as before"
    )]
    pub use_timestamp: Option<bool>,

    #[http_body_raw(description = "Rows")]
    pub body: RawDataTyped<Vec<BaseDbRowContract>>,
}

#[derive(MyHttpInput)]
pub struct BulkInsertOrReplaceIfNewInputContract {
    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,

    #[http_query(name = "syncPeriod"; description = "Synchronization period"; default)]
    pub sync_period: DataSynchronizationPeriod,

    #[http_body_raw(
        description = "Rows. Each row must carry a TimeStamp - a row is written only when it is new or its TimeStamp is greater than the stored one"
    )]
    pub body: RawDataTyped<Vec<BaseDbRowContract>>,
}

// ---- Tables ----

#[derive(MyHttpInput)]
pub struct CleanTableContract {
    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,
    #[http_query(name: "syncPeriod"; description: "Synchronization period")]
    pub sync_period: DataSynchronizationPeriod,
}

#[derive(MyHttpInput)]
pub struct UpdatePersistTableContract {
    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,

    #[http_query(description = "Persist table"; default: true)]
    pub persist: bool,
}

#[derive(MyHttpInput)]
pub struct UpdateCompressedTableContract {
    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,

    #[http_query(description = "Keep the rows of this table compressed in memory"; default: false)]
    pub compressed: bool,

    #[http_query(name: "forceCompress"; description = "If true - immediately re-encode all already stored rows to match the flag (otherwise only new/rewritten rows are affected)"; default: false)]
    pub force_compress: bool,
}

#[derive(MyHttpInput)]
pub struct CreateTableContract {
    #[http_query(name: "tableName"; description: "Name of a table")]
    pub table_name: String,

    #[http_query(description: "Persist table"; default: true)]
    pub persist: bool,

    #[http_query(name: "maxPartitionsAmount"; description: "Maximum partitions amount. Empty - means unlimited")]
    pub max_partitions_amount: Option<usize>,

    #[http_query(name: "maxRowsPerPartitionAmount"; description: "Maximum rows per partition amount. Empty - means unlimited")]
    pub max_rows_per_partition_amount: Option<usize>,

    #[http_query(description: "Keep the rows of this table compressed in memory"; default: false)]
    pub compressed: bool,

    #[http_query(name: "syncPeriod"; description: "Synchronization period"; default)]
    pub sync_period: DataSynchronizationPeriod,
}

#[derive(MyHttpInput)]
pub struct DeleteTableContract {
    #[http_header(name = "ns"; description = "Namespace to work in. Empty or absent means the default namespace")]
    pub namespace: Option<String>,
    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,
    #[http_header(name = "apikey"; description = "Api Key protecting the table to be deleted")]
    pub api_key: String,
}

/// The main node's `TableContract` - named apart from the node's own one, which lists tables by
/// name only.
#[derive(Deserialize, Serialize, MyHttpObjectStructure)]
pub struct MainNodeTableContract {
    pub name: String,
    pub persist: bool,
    #[serde(rename = "maxPartitionsAmount")]
    pub max_partitions_amount: Option<usize>,
    #[serde(rename = "maxRowsPerPartitionAmount")]
    pub max_rows_per_partition_amount: Option<usize>,
    pub compressed: bool,
}

// ---- Garbage collector ----

#[derive(MyHttpInput)]
pub struct CleanAndKeepMaxPartitionsAmountInputContract {
    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,

    #[http_query(name = "syncPeriod"; description = "Synchronization period"; default)]
    pub sync_period: DataSynchronizationPeriod,

    #[http_query(name = "maxAmount"; description = "After operations there will be no more than maxPartitionsAmount partitions")]
    pub max_partitions_amount: usize,
}

#[derive(MyHttpInput)]
pub struct CleanPartitionAndKeepMaxRowsAmountInputContract {
    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,

    #[http_query(name = "partitionKey"; description = "Partition which is going to cleaned")]
    pub partition_key: String,

    #[http_query(name = "syncPeriod"; description = "Synchronization period"; default)]
    pub sync_period: DataSynchronizationPeriod,

    #[http_query(name = "maxAmount"; description = "After operations there will be no more than maxPartitionsAmount partitions")]
    pub max_amount: usize,
}

// ---- Transactions ----

#[derive(MyHttpInput)]
pub struct ProcessTransactionInputModel {
    #[http_query(name = "transactionId" description = "Id of transaction")]
    pub transaction_id: String,

    #[http_body_raw(description = "Process transaction")]
    pub body: RawDataTyped<JsonBaseTransaction>,
}

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct JsonBaseTransaction {
    #[serde(rename = "type")]
    pub transaction_type: String,
}

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct StartTransactionResponse {
    #[serde(rename = "transactionId")]
    pub transaction_id: String,
}

// ---- Writer ping ----

#[derive(MyHttpInput)]
pub struct PingHttpInputModel {
    #[http_body(name = "name", description = "Client Name")]
    pub name: String,
    #[http_body(name = "version", description = "Client Version")]
    pub version: String,

    #[http_body(name = "tables", description = "List of tables with")]
    pub tables: Vec<String>,

    #[http_header(
        name = "session",
        description = "Writer session id issued by a previous ping"
    )]
    pub session_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct PingResult {
    #[serde(rename = "session")]
    pub session: String,
}
