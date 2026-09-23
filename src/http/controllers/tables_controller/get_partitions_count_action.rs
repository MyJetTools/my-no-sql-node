use super::models::GetPartitionsAmountContract;
use crate::app::AppContext;
use my_http_server::macros::*;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use std::{result::Result, sync::Arc};

#[http_route(
    method: "GET",
    route: "/api/Tables/PartitionsCount",
    deprecated_routes: ["/Tables/PartitionsCount"],
    input_data: "GetPartitionsAmountContract",
    description: "Get Partitions amount of selected table",
    summary: "Returns Partitions amount of selected table",
    controller: "Tables",
    result:[
        {status_code: 200, description: "Partitions amount", model: "Long"},
        {status_code: 400, description: "Table not found"},
    ]
)]
pub struct GetTablePartitionsCountAction {
    app: Arc<AppContext>,
}

impl GetTablePartitionsCountAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &GetTablePartitionsCountAction,
    input_data: GetPartitionsAmountContract,
    ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let namespace = crate::http::get_request_namespace(&action.app, ctx)?;

    let db_table =
        crate::db_operations::read::get_table(&namespace, input_data.table_name.as_str())?;

    HttpOutput::as_text(db_table.get_partitions_amount().to_string()).into_ok_result(true)
}
