use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};

use crate::app::AppContext;
use super::super::models::*;

#[http_route(
    method: "DELETE",
    route: "/api/Row",
    deprecated_routes: ["/Row"],
    controller: "Row",
    description: "Delete Entity",
    summary: "Delete Entity",
    input_data: "DeleteRowInputModel",
    result:[
        {status_code: 200, description: "Deleted row",  model:"BaseDbRowContract"},
    ],
)]
pub struct DeleteRowAction {
    app: Arc<AppContext>,
}

impl DeleteRowAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

/// Forwarded to the main node - validated first the way the main node would.
async fn handle_request(
    action: &DeleteRowAction,
    _input_data: DeleteRowInputModel,
    ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    crate::main_server_http::forward(&action.app, ctx).await
}
