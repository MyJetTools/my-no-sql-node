use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};

use crate::app::AppContext;
use super::super::models::*;

#[http_route(
    method: "POST",
    route: "/api/Bulk/CleanAndBulkInsertByChunksCommit",
    input_data: "BulkProcessInputContract",
    summary: "Commits a chunked clean-and-bulk-insert operation",
    description: "Cleans the table (or the partition the process was started with) and inserts all the accumulated rows as a single atomic operation",
    controller: "Bulk",
    result:[
        {status_code: 202, description: "Successful operation"},
        {status_code: 400, description: "Table not found"},
        {status_code: 404, description: "Process not found - it expired, was committed or the server was restarted"},
        {status_code: 409, description: "Process belongs to another writer session"},
    ],
)]
pub struct CleanAndBulkInsertByChunksCommitAction {
    app: Arc<AppContext>,
}

impl CleanAndBulkInsertByChunksCommitAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

/// Forwarded to the main node - validated first the way the main node would.
async fn handle_request(
    action: &CleanAndBulkInsertByChunksCommitAction,
    _input_data: BulkProcessInputContract,
    ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    crate::main_server_http::forward(&action.app, ctx).await
}
