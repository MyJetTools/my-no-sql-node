use my_http_server::macros::*;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use my_no_sql_sdk::core::rust_extensions::date_time::DateTimeAsMicroseconds;

use super::models::IsAliveResponse;

#[http_route(
    method: "GET",
    route: "/api/IsAlive",
    controller: "Monitoring",
    description: "Returns model shows that service is alive",
    summary: "Returns model shows that service is alive",
    result:[
        {status_code: 200, description: "Monitoring result", model: "IsAliveResponse"},
    ]
)]
pub struct IsAliveAction;

async fn handle_request(
    _: &IsAliveAction,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let response = IsAliveResponse {
        name: "MyNoSqlNode".to_string(),
        time: DateTimeAsMicroseconds::now().to_rfc3339(),
        version: crate::app::APP_VERSION.to_string(),
        env_info: std::env::var("ENV_INFO").ok(),
    };

    HttpOutput::as_json(response).into_ok_result(false)
}
