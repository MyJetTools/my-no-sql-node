use my_http_server::macros::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct NamespaceContract {
    pub name: String,
    #[serde(rename = "tablesAmount")]
    pub tables_amount: usize,
    #[serde(rename = "connectedToMainNode")]
    pub connected_to_main_node: bool,
    #[serde(rename = "mainNodePing")]
    pub main_node_ping: i64,
}
