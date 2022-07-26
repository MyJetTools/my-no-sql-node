use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use my_http_server_controllers::controllers::{
    actions::PostAction,
    documentation::{out_results::HttpResult, HttpActionDescription},
};

use crate::app::AppContext;

use super::models::{DataReaderGreetingInputModel, DataReaderGreetingResult};

pub struct GreetingAction {
    app: Arc<AppContext>,
}

impl GreetingAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}
#[async_trait::async_trait]
impl PostAction for GreetingAction {
    fn get_route(&self) -> &str {
        "/DataReader/Greeting"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: super::consts::CONTROLLER_NAME,
            description: "Issue session for http data reader",

            input_params: DataReaderGreetingInputModel::get_input_params().into(),
            results: vec![HttpResult {
                http_code: 200,
                nullable: true,
                description: "Successful operation".to_string(),
                data_type: DataReaderGreetingResult::get_http_data_structure()
                    .into_http_data_type_object(),
            }],
        }
        .into()
    }

    async fn handle_request(&self, ctx: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let http_input = DataReaderGreetingInputModel::parse_http_input(ctx).await?;

        let result = self
            .app
            .data_readers
            .add_http(ctx.request.get_ip().get_real_ip().to_string())
            .await;

        result
            .set_name_as_reader(format!("{}:{}", http_input.name, http_input.version))
            .await;

        let response = DataReaderGreetingResult {
            session_id: result.id.to_string(),
        };

        HttpOutput::as_json(response).into_ok_result(true).into()
    }
}
