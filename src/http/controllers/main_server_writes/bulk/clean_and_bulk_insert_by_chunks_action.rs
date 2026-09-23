use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};

use crate::app::AppContext;
use super::super::models::*;

#[http_route(
    method: "POST",
    route: "/api/Bulk/CleanAndBulkInsertByChunks",
    input_data: "CleanAndBulkInsertByChunksInputContract",
    summary: "Uploads a chunk of a clean-and-bulk-insert operation",
    description: "Accumulates rows aside from the table. Call it without processId to start a new process - the issued processId has to be sent with every following chunk and with the commit. Nothing is applied to the table until CleanAndBulkInsertByChunksCommit is called",
    controller: "Bulk",
    result:[
        {status_code: 200, description: "Chunk is accepted", model: "BulkProcessResponse"},
        {status_code: 400, description: "Table not found"},
        {status_code: 404, description: "Process not found - it expired, was committed or the server was restarted"},
        {status_code: 409, description: "Process belongs to another writer session or to another table"},
    ],
)]
pub struct CleanAndBulkInsertByChunksAction {
    app: Arc<AppContext>,
}

impl CleanAndBulkInsertByChunksAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

/// Forwarded to the main node - validated first the way the main node would.
async fn handle_request(
    action: &CleanAndBulkInsertByChunksAction,
    _input_data: CleanAndBulkInsertByChunksInputContract,
    ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    crate::main_server_http::forward(&action.app, ctx).await
}
