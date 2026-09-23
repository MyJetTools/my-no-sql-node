use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};

use crate::app::AppContext;
use super::super::models::*;

#[http_route(
    method: "POST",
    route: "/api/Bulk/InsertOrReplaceIfNewByChunks",
    input_data: "InsertOrReplaceIfNewByChunksInputContract",
    summary: "Uploads a chunk of an insert-or-replace-if-new operation",
    description: "Accumulates rows aside from the table. Call it without processId to start a new process - the issued processId has to be sent with every following chunk and with the commit. Nothing is applied to the table until InsertOrReplaceIfNewByChunksCommit is called. Each row must carry its own TimeStamp",
    controller: "Bulk",
    result:[
        {status_code: 200, description: "Chunk is accepted", model: "BulkProcessResponse"},
        {status_code: 400, description: "Table not found"},
        {status_code: 404, description: "Process not found - it expired, was committed or the server was restarted"},
        {status_code: 409, description: "Process belongs to another writer session or to another table"},
    ],
)]
pub struct InsertOrReplaceIfNewByChunksAction {
    app: Arc<AppContext>,
}

impl InsertOrReplaceIfNewByChunksAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

/// Forwarded to the main node - validated first the way the main node would.
async fn handle_request(
    action: &InsertOrReplaceIfNewByChunksAction,
    _input_data: InsertOrReplaceIfNewByChunksInputContract,
    ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    crate::main_server_http::forward(&action.app, ctx).await
}
