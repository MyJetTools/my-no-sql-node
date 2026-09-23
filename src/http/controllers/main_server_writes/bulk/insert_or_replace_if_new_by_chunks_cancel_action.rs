use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};

use crate::app::AppContext;
use super::super::models::*;

#[http_route(
    method: "POST",
    route: "/api/Bulk/InsertOrReplaceIfNewByChunksCancel",
    input_data: "CancelBulkProcessInputContract",
    summary: "Cancels a chunked insert-or-replace-if-new operation",
    description: "Drops the accumulated rows. The table is not touched",
    controller: "Bulk",
    result:[
        {status_code: 202, description: "Process is canceled"},
        {status_code: 404, description: "Process not found"},
        {status_code: 409, description: "Process belongs to another writer session"},
    ],
)]
pub struct InsertOrReplaceIfNewByChunksCancelAction {
    app: Arc<AppContext>,
}

impl InsertOrReplaceIfNewByChunksCancelAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

/// Forwarded to the main node - validated first the way the main node would.
async fn handle_request(
    action: &InsertOrReplaceIfNewByChunksCancelAction,
    _input_data: CancelBulkProcessInputContract,
    ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    crate::main_server_http::forward(&action.app, ctx).await
}
