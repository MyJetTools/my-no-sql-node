use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use my_http_server_controllers::controllers::{
    actions::PostAction,
    documentation::{data_types::HttpDataType, out_results::HttpResult, HttpActionDescription},
};

use crate::{
    app::AppContext,
    http::http_sessions::{self, *},
};

use super::models::SubscribeToTableInputModel;

pub struct SubscribeAction {
    app: Arc<AppContext>,
}

impl SubscribeAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl PostAction for SubscribeAction {
    fn get_route(&self) -> &str {
        "/DataReader/Subscribe"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: super::consts::CONTROLLER_NAME,
            description: "Subscribe to table",

            input_params: SubscribeToTableInputModel::get_input_params().into(),
            results: vec![
                HttpResult {
                    http_code: 202,
                    nullable: true,
                    description: "Successful operation".to_string(),
                    data_type: HttpDataType::None,
                },
                http_sessions::session_not_found_result_description(),
            ],
        }
        .into()
    }

    async fn handle_request(&self, ctx: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let input_data = SubscribeToTableInputModel::parse_http_input(ctx).await?;

        let data_reader = self
            .app
            .get_http_session(input_data.session_id.as_str())
            .await?;

        crate::operations::data_readers::subscribe(
            self.app.as_ref(),
            data_reader,
            input_data.table_name.as_str(),
        )
        .await?;

        HttpOutput::Empty.into_ok_result(true).into()
    }
}
