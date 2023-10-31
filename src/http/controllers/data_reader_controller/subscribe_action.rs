use my_http_server::macros::*;
use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use crate::{app::AppContext, http::http_sessions::*};

use super::models::SubscribeToTableInputModel;

#[http_route(
    method: "POST",
    route: "/DataReader/Subscribe",
    controller: "DataReader",
    summary: "Subscribes to table",
    description: "Subscribe to table",
    input_data: "SubscribeToTableInputModel",
    result:[
        {status_code: 202, description: "Successful operation"},
    ]
)]
pub struct SubscribeAction {
    app: Arc<AppContext>,
}

impl SubscribeAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &SubscribeAction,
    input_data: SubscribeToTableInputModel,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let data_reader = action
        .app
        .get_http_session(input_data.session_id.as_str())
        .await?;

    crate::operations::data_readers::subscribe(
        action.app.as_ref(),
        data_reader,
        input_data.table_name.as_str(),
    )
    .await?;

    HttpOutput::Empty.into_ok_result(true).into()
}
