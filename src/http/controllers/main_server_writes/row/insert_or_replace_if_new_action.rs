use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};

use crate::app::AppContext;
use super::super::models::*;

#[http_route(
    method: "POST",
    route: "/api/Row/InsertOrReplaceIfNew",
    deprecated_routes: ["/Row/InsertOrReplaceIfNew"],
    controller: "Row",
    description: "Insert or replace DbEntity only when it is newer than the stored one",
    summary: "Inserts a missing DbEntity, or replaces it only when the incoming TimeStamp is greater than the stored one",
    input_data: "InsertOrReplaceIfNewInputContract",
    result:[
        {status_code: 200, description: "Removed entities"},
    ],
)]
pub struct InsertOrReplaceIfNewAction {
    app: Arc<AppContext>,
}

impl InsertOrReplaceIfNewAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

/// Forwarded to the main node - validated first the way the main node would.
async fn handle_request(
    action: &InsertOrReplaceIfNewAction,
    _input_data: InsertOrReplaceIfNewInputContract,
    ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    crate::main_server_http::forward(&action.app, ctx).await
}
