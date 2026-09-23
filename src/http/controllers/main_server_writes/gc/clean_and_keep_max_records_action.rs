use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};

use crate::app::AppContext;
use super::super::models::*;

#[http_route(
    method: "POST",
    route: "/api/GarbageCollector/CleanAndKeepMaxRecords",
    deprecated_routes: ["/GarbageCollector/CleanAndKeepMaxRecords"],
    summary: "Makes sure we keep maximum rows amount required",
    description: "After operation some rows are going to be deleted to make sure we keep maximum rows amount required",
    controller: "GarbageCollector",
    input_data: "CleanPartitionAndKeepMaxRowsAmountInputContract",
    result:[
        {status_code: 202, description: "Successful operation"},
        {status_code: 400, description: "Table not found"}
    ],
)]
pub struct CleanPartitionAndKepMaxRecordsControllerAction {
    app: Arc<AppContext>,
}

impl CleanPartitionAndKepMaxRecordsControllerAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

/// Forwarded to the main node - validated first the way the main node would.
async fn handle_request(
    action: &CleanPartitionAndKepMaxRecordsControllerAction,
    _input_data: CleanPartitionAndKeepMaxRowsAmountInputContract,
    ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    crate::main_server_http::forward(&action.app, ctx).await
}
