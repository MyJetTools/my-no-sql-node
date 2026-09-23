use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};

use crate::app::AppContext;
use super::super::models::*;

#[http_route(
    method: "POST",
    route: "/api/Row/Insert",
    deprecated_routes: ["/Row/Insert"],
    controller: "Row",
    description: "Insert Row",
    summary: "Inserts Row",
    input_data: "InsertInputContract",
    result:[
        {status_code: 200, description: "Amount of rows of the table or the partition"},
    ],
)]
pub struct InsertRowAction {
    app: Arc<AppContext>,
}

impl InsertRowAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

/// Forwarded to the main node - validated first the way the main node would.
async fn handle_request(
    action: &InsertRowAction,
    _input_data: InsertInputContract,
    ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    crate::main_server_http::forward(&action.app, ctx).await
}
