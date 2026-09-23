use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};

use crate::app::AppContext;
use super::super::models::*;

#[http_route(
    method: "POST",
    route: "/api/Ping",
    controller: "Monitoring",
    description: "Endpoint to ping the service",
    summary: "Endpoint to ping the service",
    input_data: PingHttpInputModel,
    result:[
        {status_code: 200, description: "Issued/refreshed writer session", model: "PingResult"},
    ],
)]
pub struct PingAction {
    app: Arc<AppContext>,
}

impl PingAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

/// Forwarded to the main node - validated first the way the main node would.
async fn handle_request(
    action: &PingAction,
    _input_data: PingHttpInputModel,
    ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    crate::main_server_http::forward(&action.app, ctx).await
}
