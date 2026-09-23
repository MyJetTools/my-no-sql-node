use my_http_server::macros::*;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput, WebContentType};
use std::sync::Arc;

use crate::app::AppContext;

#[http_route(
    method: "GET",
    route: "/metrics",
)]
pub struct MetricsAction {
    app: Arc<AppContext>,
}

impl MetricsAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &MetricsAction,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    HttpOutput::from_builder()
        .set_content_type(WebContentType::Text)
        .set_content(action.app.metrics.build().into_bytes())
        .into_ok_result(false)
}
