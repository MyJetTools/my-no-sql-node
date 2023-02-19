use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};

use crate::{app::AppContext, http::controllers::row_controller::models::BaseDbRowContract};

use super::models::NextMultipartRequestInputContract;

#[my_http_server_swagger::http_route(
    method: "POST",
    route: "/Multipart/Next",
    controller: "Multipart",
    description: "New multipart request is started",
    summary: "Returns first multipart amount of rows",
    input_data: "NextMultipartRequestInputContract",
    result:[
        {status_code: 200, description: "Chunk of entities", model: "Vec<BaseDbRowContract>" },
    ]
)]
pub struct NextMultipartController {
    app: Arc<AppContext>,
}

impl NextMultipartController {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &NextMultipartController,
    input_data: NextMultipartRequestInputContract,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let db_rows = crate::db_operations::read::multipart::get_next(
        action.app.as_ref(),
        input_data.request_id,
        input_data.max_records_count,
    )
    .await;

    if db_rows.is_none() {
        return Err(HttpFailResult::as_not_found("".to_string(), false));
    }

    return Ok(db_rows.unwrap().into());
}
