use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};

use crate::app::AppContext;
use super::super::models::*;

#[http_route(
    method: "POST",
    route: "/api/Bulk/Delete",
    deprecated_routes: ["/Bulk/Delete"],
    input_data: "BulkDeleteInputContract",
    summary: "Bulk delete operation",
    description: "Does bulk delete operation",
    controller: "Bulk",
    result:[
        {status_code: 202, description: "Successful operation"},
        {status_code: 404, description: "Table not found"},
    ],
)]
pub struct BulkDeleteAction {
    app: Arc<AppContext>,
}

impl BulkDeleteAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

/// Forwarded to the main node - validated first the way the main node would.
async fn handle_request(
    action: &BulkDeleteAction,
    _input_data: BulkDeleteInputContract,
    ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    crate::main_server_http::forward(&action.app, ctx).await
}
