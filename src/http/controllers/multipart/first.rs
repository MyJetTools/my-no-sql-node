use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use my_http_server_controllers::controllers::{
    actions::PostAction,
    documentation::{out_results::IntoHttpResult, HttpActionDescription},
};

use crate::app::AppContext;

use super::models::{NewMultipartInputContract, NewMultipartResponse};

pub struct FirstMultipartController {
    app: Arc<AppContext>,
}

impl FirstMultipartController {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl PostAction for FirstMultipartController {
    fn get_route(&self) -> &str {
        "/Multipart/First"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: super::consts::CONTROLLER_NAME,
            description: "Monitoring API",

            input_params: Some(NewMultipartInputContract::get_input_params()),
            results: vec![
                NewMultipartResponse::get_http_data_structure().into_http_result_object(
                    200,
                    false,
                    "New multipart is started",
                ),
            ],
        }
        .into()
    }
    async fn handle_request(&self, ctx: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let input_data = NewMultipartInputContract::parse_http_input(ctx).await?;

        let result = crate::db_operations::read::multipart::start_read_all(
            self.app.as_ref(),
            input_data.table_name.as_ref(),
        )
        .await?;

        let response = NewMultipartResponse {
            snapshot_id: format!("{}", result),
        };

        HttpOutput::as_json(response).into_ok_result(true).into()
    }
}
