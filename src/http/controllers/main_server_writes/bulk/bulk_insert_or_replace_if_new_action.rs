use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};

use crate::app::AppContext;
use super::super::models::*;

#[http_route(
    method: "POST",
    route: "/api/Bulk/InsertOrReplaceIfNew",
    deprecated_routes: ["/Bulk/InsertOrReplaceIfNew"],
    input_data: "BulkInsertOrReplaceIfNewInputContract",

    summary: "Bulk insert or replace if the row is newer",
    description: "Bulk operation that inserts a row when it is missing, or replaces it only when the incoming TimeStamp is greater than the stored one",
    controller: "Bulk",
    result:[
        {status_code: 202, description: "Successful operation"},
        {status_code: 404, description: "Table not found"},
    ],
)]
pub struct BulkInsertOrReplaceIfNewAction {
    app: Arc<AppContext>,
}

impl BulkInsertOrReplaceIfNewAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

/// Forwarded to the main node - validated first the way the main node would.
async fn handle_request(
    action: &BulkInsertOrReplaceIfNewAction,
    _input_data: BulkInsertOrReplaceIfNewInputContract,
    ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    crate::main_server_http::forward(&action.app, ctx).await
}
