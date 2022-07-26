use std::sync::Arc;

use crate::app::AppContext;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use my_http_server_controllers::controllers::{
    actions::GetAction,
    documentation::{out_results::IntoHttpResult, HttpActionDescription},
};

use super::models::StatusModel;

pub struct StatusController {
    app: Arc<AppContext>,
}

impl StatusController {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl GetAction for StatusController {
    fn get_route(&self) -> &str {
        "/Api/Status"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: "Monitoring",
            description: "Monitoring API",

            input_params: None,
            results: vec![
                StatusModel::get_http_data_structure().into_http_result_object(
                    200,
                    false,
                    "Monitoring result",
                ),
            ],
        }
        .into()
    }

    async fn handle_request(&self, _ctx: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let model = StatusModel::new(self.app.as_ref()).await;
        HttpOutput::as_json(model).into_ok_result(true).into()
    }
}
