use my_http_server_swagger::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct IsAliveResponse {
    pub name: String,
    pub time: String,
    pub version: String,
    pub env_info: Option<String>,
}
