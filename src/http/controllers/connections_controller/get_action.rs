use crate::app::AppContext;
use my_http_server::macros::*;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use rest_api_shared::ConnectionsContract;
use std::sync::Arc;

#[http_route(
    method: "GET",
    route: "/api/Connections",
    controller: "Monitoring",
    description: "Connections traffic metrics",
    summary: "Returns the traffic of every reader and of every connection to the main node",
    result:[
        {status_code: 200, description: "Connections traffic snapshot", model: "ConnectionsContract"},
    ]
)]
pub struct GetConnectionsAction {
    app: Arc<AppContext>,
}

impl GetConnectionsAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &GetConnectionsAction,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let result: ConnectionsContract = super::models::build_connections(action.app.as_ref());
    HttpOutput::as_json(result).into_ok_result(false)
}
