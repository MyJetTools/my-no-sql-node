use crate::app::AppContext;
use my_http_server::macros::*;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use rest_api_shared::StatusModel;
use std::sync::Arc;

#[http_route(
    method: "GET",
    route: "/api/Status",
    controller: "Monitoring",
    description: "Monitoring API",
    summary: "Returns monitoring metrics",
    result:[
        {status_code: 200, description: "Monitoring snapshot", model: "StatusModel"},
    ]
)]
pub struct StatusAction {
    app: Arc<AppContext>,
}

impl StatusAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &StatusAction,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    HttpOutput::as_json(super::models::build_status_model(action.app.as_ref())).into_ok_result(false)
}
