use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use crate::app::AppContext;

use super::models::{DataReaderGreetingInputModel, DataReaderGreetingResult};

#[my_http_server_swagger::http_route(
    method: "POST",
    route: "/DataReader/Greeting",
    controller: "DataReader",
    summary: "Issue session for http data reader",
    description: "Issues session for http data reader",
    input_data: "DataReaderGreetingInputModel",
    result:[
        {status_code: 200, description: "Successful operation"},
    ]
)]
pub struct GreetingAction {
    app: Arc<AppContext>,
}

impl GreetingAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &GreetingAction,
    http_input: DataReaderGreetingInputModel,
    ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let result = action
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
