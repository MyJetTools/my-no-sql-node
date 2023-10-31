use crate::app::AppContext;
use my_http_server::macros::*;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use std::sync::Arc;

use super::models::StatusModel;

#[http_route(
    method: "GET",
    route: "/Api/Status",
    controller: "Monitoring",
    description: "Monitoring API",
    summary: "Returns monitoring metrics",
    result:[
        {status_code: 200, description: "Monitoring snapshot", model_as_array: "StatusModel"},
    ]
)]
pub struct StatusController {
    app: Arc<AppContext>,
}

impl StatusController {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

/*
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


}
 */

async fn handle_request(
    action: &StatusController,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let model = StatusModel::new(action.app.as_ref()).await;
    HttpOutput::as_json(model).into_ok_result(true).into()
}
