use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};

use crate::app::AppContext;
use super::super::models::*;

#[http_route(
    method: "PUT",
    route: "/api/Row/Replace",
    deprecated_routes: ["/Row/Replace"],
    controller: "Row",
    description: "Replace Entity",
    summary: "Replaces Entity",
    input_data: "ReplaceInputContract",
    result:[
        {status_code: 200, description: "Replaced row",  model:"BaseDbRowContract"},
    ],
)]
pub struct ReplaceRowAction {
    app: Arc<AppContext>,
}

impl ReplaceRowAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

/// Forwarded to the main node - validated first the way the main node would.
async fn handle_request(
    action: &ReplaceRowAction,
    _input_data: ReplaceInputContract,
    ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    crate::main_server_http::forward(&action.app, ctx).await
}
