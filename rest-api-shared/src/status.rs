use my_http_utils::macros::MyHttpObjectStructure;
use serde::{Deserialize, Serialize};

/// `GET /api/Status`. The node serves from the start, so `not_initialized` is always `false` -
/// it stays in the contract the main node has.
#[derive(Serialize, Deserialize, MyHttpObjectStructure, Clone, Debug, PartialEq)]
pub struct StatusModel {
    #[serde(rename = "notInitialized")]
    pub not_initialized: bool,
    pub initialized: InitializedModel,
    #[serde(rename = "statusBar")]
    pub status_bar: StatusBarModel,
}

#[derive(Serialize, Deserialize, MyHttpObjectStructure, Clone, Debug, PartialEq)]
pub struct InitializedModel {
    pub readers: Vec<ReaderModel>,
    pub tables: Vec<TableModel>,
    pub namespaces: Vec<NamespaceStatusModel>,
}

/// `TableModel::sync_state` - the table has arrived from the main node, which pushes every change
/// of it to the node.
pub const TABLE_SYNC_REPLICATED: &str = "replicated";
/// `TableModel::sync_state` - a reader subscribed to the table, the node asked the main node for
/// it, and nothing has arrived yet.
pub const TABLE_SYNC_PENDING: &str = "pending";
/// `TableModel::sync_state` - the main node does not have the table. The node serves it empty and
/// asks for it again from time to time.
pub const TABLE_SYNC_NOT_FOUND: &str = "notFound";

/// A table of the node. A node has no tables of its own: each one is here because a reader
/// subscribed to it, and it is replicated from the main node from then on.
#[derive(Serialize, Deserialize, MyHttpObjectStructure, Clone, Debug, PartialEq)]
pub struct TableModel {
    pub namespace: String,
    pub name: String,
    #[serde(rename = "syncState")]
    pub sync_state: String,
    #[serde(rename = "partitionsCount")]
    pub partitions_count: u64,
    #[serde(rename = "dataSize")]
    pub data_size: u64,
    #[serde(rename = "recordsAmount")]
    pub records_amount: u64,
}

#[derive(Serialize, Deserialize, MyHttpObjectStructure, Clone, Debug, PartialEq)]
pub struct ReaderModel {
    pub id: String,
    pub name: String,
    pub namespace: String,
    pub ip: String,
    pub tables: Vec<String>,
    #[serde(rename = "awaitingTables")]
    pub awaiting_tables: Vec<String>,
    #[serde(rename = "lastIncomingTime")]
    pub last_incoming_time: String,
    #[serde(rename = "connectedTime")]
    pub connected_time: String,
    #[serde(rename = "pendingToSend")]
    pub pending_to_send: u64,
    #[serde(rename = "sentPerSecond")]
    pub sent_per_second: Vec<u64>,
}

#[derive(Serialize, Deserialize, MyHttpObjectStructure, Clone, Debug, PartialEq)]
pub struct NamespaceStatusModel {
    pub name: String,
    #[serde(rename = "connectedToMainNode")]
    pub connected_to_main_node: bool,
    #[serde(rename = "mainNodePing")]
    pub main_node_ping: i64,
}

#[derive(Serialize, Deserialize, MyHttpObjectStructure, Clone, Debug, PartialEq)]
pub struct LocationModel {
    pub id: String,
    pub compress: bool,
}

#[derive(Serialize, Deserialize, MyHttpObjectStructure, Clone, Debug, PartialEq)]
pub struct StatusBarModel {
    pub location: LocationModel,
    #[serde(rename = "version")]
    pub version: String,
    #[serde(rename = "tcpConnections")]
    pub tcp_connections: u64,
    #[serde(rename = "tablesAmount")]
    pub tables_amount: u64,
    #[serde(rename = "httpConnections")]
    pub http_connections: u64,
    #[serde(rename = "connectedToMainNode")]
    pub connected_to_main_node: bool,
    #[serde(rename = "mainNodePing")]
    pub main_node_ping: i64,
}
