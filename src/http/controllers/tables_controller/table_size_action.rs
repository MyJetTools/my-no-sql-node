use my_http_server::macros::*;
use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use crate::app::AppContext;

use super::models::GetTableSizeContract;

#[http_route(
    method: "GET",
    route: "/api/Tables/TableSize",
    deprecated_routes: ["/Tables/TableSize"],
    input_data: "GetTableSizeContract",
    description: "Get Table size",
    summary: "Returns Table size",
    controller: "Tables",
    result:[
        {status_code: 200, description: "Size of table", model: "Long"},
        {status_code: 400, description: "Table not found"},
    ]
)]
pub struct GetTableSizeAction {
    app: Arc<AppContext>,
}

impl GetTableSizeAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &GetTableSizeAction,
    input_data: GetTableSizeContract,
    ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let namespace = crate::http::get_request_namespace(&action.app, ctx)?;

    let db_table =
        crate::db_operations::read::get_table(&namespace, input_data.table_name.as_str())?;

    HttpOutput::as_text(db_table.get_table_size().to_string()).into_ok_result(true)
}
