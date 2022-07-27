use std::sync::atomic::Ordering;

use crate::app::AppContext;
use my_http_server_swagger::*;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct LocationModel {
    pub id: String,
    pub compress: bool,
}

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct StatusBarModel {
    pub location: LocationModel,
    #[serde(rename = "tcpConnections")]
    pub tcp_connections: usize,
    #[serde(rename = "tablesAmount")]
    pub tables_amount: usize,
    #[serde(rename = "httpConnections")]
    pub http_connections: usize,
    #[serde(rename = "connectedToMainNode")]
    pub connected_to_main_node: bool,
    #[serde(rename = "mainNodePing")]
    pub main_node_ping: i64,
}

impl StatusBarModel {
    pub fn new(
        app: &AppContext,
        tcp_connections: usize,
        http_connections: usize,
        tables_amount: usize,
    ) -> Self {
        Self {
            location: LocationModel {
                id: app.settings.location.to_string(),
                compress: false,
            },
            tcp_connections,
            http_connections,
            tables_amount,
            main_node_ping: app.master_node_ping_interval.load(Ordering::Relaxed),
            connected_to_main_node: app.connected_to_main_node.has_connection(),
        }
    }
}
