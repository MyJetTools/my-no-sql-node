use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};

use crate::app::AppContext;
use super::super::models::*;

#[http_route(
    method: "POST",
    route: "/api/GarbageCollector/CleanAndKeepMaxPartitions",
    deprecated_routes: ["/GarbageCollector/CleanAndKeepMaxPartitions"],
    summary: "Makes sure we keep maximum partitions amount required",
    description: "After operation some partitions can be deleted to make sure we keep maximum partitions amount required",
    controller: "GarbageCollector",
    input_data: CleanAndKeepMaxPartitionsAmountInputContract,
    result:[
        {status_code: 202, description: "Successful operation"},
        {status_code: 400, description: "Table not found"}
    ],
)]
pub struct CleanAndKeepMaxPartitionsAmountAction {
    app: Arc<AppContext>,
}

impl CleanAndKeepMaxPartitionsAmountAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

/// Forwarded to the main node - validated first the way the main node would.
async fn handle_request(
    action: &CleanAndKeepMaxPartitionsAmountAction,
    _input_data: CleanAndKeepMaxPartitionsAmountInputContract,
    ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    crate::main_server_http::forward(&action.app, ctx).await
}
