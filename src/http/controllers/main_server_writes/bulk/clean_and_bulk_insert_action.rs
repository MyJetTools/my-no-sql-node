use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};

use crate::app::AppContext;
use super::super::models::*;

#[http_route(
    method: "POST",
    route: "/api/Bulk/CleanAndBulkInsert",
    deprecated_routes: ["/Bulk/CleanAndBulkInsert"],
    input_data: "CleanAndBulkInsertInputContract",
    summary: "Cleans partition and does bulk insert operation as a single transaction",
    description: "Cleans partition and does bulk insert operation as a single transaction",
    controller: "Bulk",
    result:[
        {status_code: 202, description: "Successful operation"},
        {status_code: 404, description: "Table not found"},
    ],
)]
pub struct CleanAndBulkInsertAction {
    app: Arc<AppContext>,
}

impl CleanAndBulkInsertAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

/// Forwarded to the main node - validated first the way the main node would.
async fn handle_request(
    action: &CleanAndBulkInsertAction,
    _input_data: CleanAndBulkInsertInputContract,
    ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    crate::main_server_http::forward(&action.app, ctx).await
}
