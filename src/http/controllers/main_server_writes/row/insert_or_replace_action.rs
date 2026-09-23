use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};

use crate::app::AppContext;
use super::super::models::*;

#[http_route(
    method: "POST",
    route: "/api/Row/InsertOrReplace",
    deprecated_routes: ["/Row/InsertOrReplace"],
    controller: "Row",
    description: "Insert or replace DbEntity",
    summary: "Inserts or replaces DbEntity",
    input_data: "InsertOrReplaceInputContract",
    result:[
        {status_code: 200, description: "Removed entities"},
    ],
)]
pub struct InsertOrReplaceAction {
    app: Arc<AppContext>,
}

impl InsertOrReplaceAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

/// Forwarded to the main node - validated first the way the main node would.
async fn handle_request(
    action: &InsertOrReplaceAction,
    _input_data: InsertOrReplaceInputContract,
    ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    crate::main_server_http::forward(&action.app, ctx).await
}
