use super::models::*;
use crate::app::AppContext;
use my_http_server::macros::*;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use std::{result::Result, sync::Arc};

#[http_route(
    method: "GET",
    route: "/api/Partitions",
    deprecated_routes: ["/Partitions"],
    input_data: "GetPartitionsListContract",
    description: "Get Partitions of selected table",
    summary: "Returns Partitions of selected table",
    controller: "Partitions",
    result:[
        {status_code: 200, description: "Partitions", model: "PartitionsHttpResult"},
        {status_code: 400, description: "Table not found"},
    ]
)]
pub struct GetPartitionsAction {
    app: Arc<AppContext>,
}

impl GetPartitionsAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &GetPartitionsAction,
    input_data: GetPartitionsListContract,
    ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let namespace = crate::http::get_request_namespace(&action.app, ctx)?;

    let db_table =
        crate::db_operations::read::get_table(&namespace, input_data.table_name.as_str())?;

    let partitions = crate::db_operations::read::partitions::get_partitions(
        &db_table,
        input_data.limit,
        input_data.skip,
    );

    let result = PartitionsHttpResult {
        amount: partitions.total_amount,
        data: partitions.partition_keys,
    };

    HttpOutput::as_json(result).into_ok_result(true)
}
