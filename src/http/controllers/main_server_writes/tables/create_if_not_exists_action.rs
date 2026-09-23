use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};

use crate::app::AppContext;
use super::super::models::*;

#[http_route(
    method: "POST",
    route: "/api/Tables/CreateIfNotExists",
    deprecated_routes: ["/Tables/CreateIfNotExists"],
    input_data: CreateTableContract,
    description: "Create table if not exists",
    summary: "Creates table if not exists",
    controller: "Tables",
    result:[
        {status_code: 200, description: "Table is created", model: "MainNodeTableContract"},
    ],
)]
pub struct CreateIfNotExistsAction {
    app: Arc<AppContext>,
}

impl CreateIfNotExistsAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

/// Forwarded to the main node - validated first the way the main node would.
async fn handle_request(
    action: &CreateIfNotExistsAction,
    _input_data: CreateTableContract,
    ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    crate::main_server_http::forward(&action.app, ctx).await
}
