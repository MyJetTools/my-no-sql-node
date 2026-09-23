use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};

use crate::app::AppContext;
use super::super::models::*;

#[http_route(
    method: "POST",
    route: "/api/Tables/UpdatePersist",
    deprecated_routes: ["/Tables/UpdatePersist"],

    input_data: "UpdatePersistTableContract",
    description: "Update table persistence state",
    summary: "Updates table persistence state",
    controller: "Tables",
    result:[
        {status_code: 202, description: "Updated succesfully"},
        {status_code: 400, description: "Table not found"},
    ],
)]
pub struct UpdatePersistAction {
    app: Arc<AppContext>,
}

impl UpdatePersistAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

/// Forwarded to the main node - validated first the way the main node would.
async fn handle_request(
    action: &UpdatePersistAction,
    _input_data: UpdatePersistTableContract,
    ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    crate::main_server_http::forward(&action.app, ctx).await
}
