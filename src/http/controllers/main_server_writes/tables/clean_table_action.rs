use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};

use crate::app::AppContext;
use super::super::models::*;

#[http_route(
    method: "PUT",
    route: "/api/Tables/Clean",
    deprecated_routes: ["/Tables/Clean"],
    input_data: "CleanTableContract",
    description: "Clean Table",
    summary: "Cleans Table",
    controller: "Tables",
    result:[
        {status_code: 202, description: "Table is cleaned"},
    ],
)]
pub struct CleanTableAction {
    app: Arc<AppContext>,
}

impl CleanTableAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

/// Forwarded to the main node - validated first the way the main node would.
async fn handle_request(
    action: &CleanTableAction,
    _input_data: CleanTableContract,
    ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    crate::main_server_http::forward(&action.app, ctx).await
}
