use my_http_utils::macros::MyHttpObjectStructure;
use serde::{Deserialize, Serialize};

/// `GET /api/Connections` - live traffic of the node: its readers, and its own connections to
/// the main node, one per namespace. Bytes per second; a reader's is `None` when the node can not
/// count it (an HTTP reader). Incoming bytes of a main node link are the table data it sent,
/// uncompressed - with `Compress` on, the bytes on the wire are far fewer.
#[derive(Serialize, Deserialize, MyHttpObjectStructure, Clone, Debug, PartialEq)]
pub struct ConnectionsContract {
    #[serde(rename = "outgoingPerSecond")]
    pub outgoing_per_second: u64,
    pub readers: Vec<ConnectionReaderContract>,
    #[serde(rename = "mainNodes")]
    pub main_nodes: Vec<MainNodeConnectionContract>,
}

#[derive(Serialize, Deserialize, MyHttpObjectStructure, Clone, Debug, PartialEq)]
pub struct ConnectionReaderContract {
    pub id: String,
    pub name: String,
    pub namespace: String,
    pub ip: String,
    pub tables: Vec<String>,
    #[serde(rename = "awaitingTables")]
    pub awaiting_tables: Vec<String>,
    #[serde(rename = "outgoingPerSecond")]
    pub outgoing_per_second: Option<u64>,
    #[serde(rename = "pendingToSend")]
    pub pending_to_send: u64,
    #[serde(rename = "lastIncomingTime")]
    pub last_incoming_time: String,
    // Round trip in microseconds, as the reader reported it. Null until it does.
    pub latency: Option<i64>,
}

#[derive(Serialize, Deserialize, MyHttpObjectStructure, Clone, Debug, PartialEq)]
pub struct MainNodeConnectionContract {
    pub namespace: String,
    pub connected: bool,
    pub ping: i64,
    #[serde(rename = "incomingPerSecond")]
    pub incoming_per_second: u64,
    #[serde(rename = "outgoingPerSecond")]
    pub outgoing_per_second: u64,
}
