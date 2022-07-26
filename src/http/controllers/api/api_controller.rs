use async_trait::async_trait;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use rust_extensions::date_time::DateTimeAsMicroseconds;

use my_http_server_controllers::controllers::{
    actions::GetAction,
    documentation::{out_results::IntoHttpResult, HttpActionDescription},
};

use super::models::IsAliveResponse;

pub struct ApiController {}

impl ApiController {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl GetAction for ApiController {
    fn get_route(&self) -> &str {
        "/Api/IsAlive"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: "Monitoring",
            description: "Monitoring API",

            input_params: None,
            results: vec![
                IsAliveResponse::get_http_data_structure().into_http_result_object(
                    200,
                    false,
                    "Monitoring result",
                ),
            ],
        }
        .into()
    }

    async fn handle_request(&self, _ctx: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
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
}
