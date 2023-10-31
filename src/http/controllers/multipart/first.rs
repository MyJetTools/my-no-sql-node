use my_http_server::macros::*;
use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use crate::app::AppContext;

use super::models::{NewMultipartInputContract, NewMultipartResponse};

#[http_route(
    method: "POST",
    route: "/Multipart/First",
    controller: "Multipart",
    description: "New multipart request is started",
    summary: "Returns first multipart amount of rows",
    input_data: "NewMultipartInputContract",
    result:[
        {status_code: 200, description: "Rows"},
    ]
)]
pub struct FirstMultipartController {
    app: Arc<AppContext>,
}

impl FirstMultipartController {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &FirstMultipartController,
    input_data: NewMultipartInputContract,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let result = crate::db_operations::read::multipart::start_read_all(
        action.app.as_ref(),
        input_data.table_name.as_ref(),
    )
    .await?;

    let response = NewMultipartResponse {
        snapshot_id: format!("{}", result),
    };

    HttpOutput::as_json(response).into_ok_result(true).into()
}
