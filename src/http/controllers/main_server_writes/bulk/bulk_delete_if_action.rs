use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};

use crate::app::AppContext;
use super::super::models::*;

#[http_route(
    method: "POST",
    route: "/api/Bulk/DeleteIf",
    input_data: "BulkDeleteIfInputContract",
    summary: "Bulk delete of the rows which are still at the given version",
    description: "Deletes a row of the batch only when the TimeStamp stored in the table is still the one sent with that row. A row which has been rewritten meanwhile - and a row which is not there any more - is left alone and reported back in the response, the rest of the batch is still deleted",
    controller: "Bulk",
    result:[
        {status_code: 200, description: "Amount of deleted rows and the ones which were left in place", model: "BulkDeleteIfResponseContract"},
        {status_code: 400, description: "Table not found, or a row of the batch carries no valid TimeStamp"},
    ],
)]
pub struct BulkDeleteIfAction {
    app: Arc<AppContext>,
}

impl BulkDeleteIfAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

/// Forwarded to the main node - validated first the way the main node would.
async fn handle_request(
    action: &BulkDeleteIfAction,
    _input_data: BulkDeleteIfInputContract,
    ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    crate::main_server_http::forward(&action.app, ctx).await
}
