use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};

use crate::app::AppContext;
use super::super::models::*;

#[http_route(
    method: "DELETE",
    route: "/api/Tables/Delete",
    deprecated_routes: ["/Tables/Delete"],
    input_data: "DeleteTableContract",
    description: "Delete Table",
    summary: "Deletes Table",
    controller: "Tables",
    result:[
        {status_code: 202, description: "Table is deleted"},
    ],
)]
pub struct DeleteTableAction {
    app: Arc<AppContext>,
}

impl DeleteTableAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

/// Forwarded to the main node - validated first the way the main node would.
async fn handle_request(
    action: &DeleteTableAction,
    _input_data: DeleteTableContract,
    ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    crate::main_server_http::forward(&action.app, ctx).await
}
