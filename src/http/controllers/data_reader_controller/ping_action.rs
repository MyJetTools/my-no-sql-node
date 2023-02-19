use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use crate::{app::AppContext, http::http_sessions::HttpSessionsSupport};

use super::models::PingInputModel;

#[my_http_server_swagger::http_route(
    method: "POST",
    route: "/DataReader/Ping",
    controller: "DataReader",
    summary: "Pings that subscriber is alive",
    description: "Pings that subscriber is alive",
    input_data: "PingInputModel",
    result:[
        {status_code: 202, description: "Successful operation"},
    ]
)]
pub struct PingAction {
    app: Arc<AppContext>,
}

impl PingAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &PingAction,
    input_data: PingInputModel,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    action
        .app
        .get_http_session(input_data.session_id.as_str())
        .await?;

    HttpOutput::Empty.into_ok_result(true).into()
}
