use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};

use crate::app::AppContext;
use super::super::models::*;

#[http_route(
    method: "POST",
    route: "/api/Tables/Create",
    deprecated_routes: ["/Tables/Create"],
    input_data: "CreateTableContract",
    description: "Create table",
    summary: "Creates table",
    controller: "Tables",
    result:[
        {status_code: 202, description: "Table is created"},
        {status_code: 400, description: "Table already exists"},
    ],
)]
pub struct CreateTableAction {
    app: Arc<AppContext>,
}

impl CreateTableAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

/// Forwarded to the main node - validated first the way the main node would.
async fn handle_request(
    action: &CreateTableAction,
    _input_data: CreateTableContract,
    ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    crate::main_server_http::forward(&action.app, ctx).await
}
