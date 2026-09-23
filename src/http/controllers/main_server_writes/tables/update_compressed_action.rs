use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};

use crate::app::AppContext;
use super::super::models::*;

#[http_route(
    method: "POST",
    route: "/api/Tables/UpdateCompressed",
    deprecated_routes: ["/Tables/UpdateCompressed"],

    input_data: "UpdateCompressedTableContract",
    description: "Update table in-memory compression state",
    summary: "Compresses or decompresses the already stored rows of the table",
    controller: "Tables",
    result:[
        {status_code: 202, description: "Updated succesfully"},
        {status_code: 400, description: "Table not found"},
    ],
)]
pub struct UpdateCompressedAction {
    app: Arc<AppContext>,
}

impl UpdateCompressedAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

/// Forwarded to the main node - validated first the way the main node would.
async fn handle_request(
    action: &UpdateCompressedAction,
    _input_data: UpdateCompressedTableContract,
    ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    crate::main_server_http::forward(&action.app, ctx).await
}
