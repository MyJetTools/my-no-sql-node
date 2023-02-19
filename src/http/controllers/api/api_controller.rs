use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use rust_extensions::date_time::DateTimeAsMicroseconds;

use super::models::IsAliveResponse;

#[my_http_server_swagger::http_route(
    method: "GET",
    route: "/Api/IsAlive",
    controller: "Monitoring",
    description: "Returns model shows that service is alive",
    summary: "Returns model shows that service is alive",
    result:[
        {status_code: 200, description: "Monitoring result", model: "IsAliveResponse"},
    ]
)]
pub struct ApiController {}

impl ApiController {
    pub fn new() -> Self {
        Self {}
    }
}

async fn handle_request(
    _: &ApiController,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let version = env!("CARGO_PKG_VERSION");

    let env_info = match std::env::var("ENV_INFO") {
        Ok(value) => Some(value),
        Err(_) => None,
    };

    let time = DateTimeAsMicroseconds::now();

    let response = IsAliveResponse {
        name: "MyNoSqlServer".to_string(),
        time: time.to_rfc3339(),
        version: version.to_string(),
        env_info,
    };

    HttpOutput::as_json(response).into_ok_result(true).into()
}
