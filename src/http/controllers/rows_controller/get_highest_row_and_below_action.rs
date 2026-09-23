use my_http_server::macros::*;
use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};

use crate::app::AppContext;

use super::models::GetHighestRowsAndBelowInputContract;

#[http_route(
    method: "GET",
    route: "/api/Rows/HighestRowAndBelow",
    deprecated_routes: ["/Rows/HighestRowAndBelow"],
    controller: "Rows",
    description: "Gets row with highest row_key and below",
    summary: "Returns row with highest row_key and below",
    input_data: "GetHighestRowsAndBelowInputContract",
    result:[
        {status_code: 200, description: "Rows"},
    ]
)]
pub struct GetHighestRowAndBelowAction {
    app: Arc<AppContext>,
}

impl GetHighestRowAndBelowAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &GetHighestRowAndBelowAction,
    input_data: GetHighestRowsAndBelowInputContract,
    ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let namespace = crate::http::get_request_namespace(&action.app, ctx)?;

    let db_table =
        crate::db_operations::read::get_table(&namespace, input_data.table_name.as_str())?;

    // Zero is "no limit", the way the main node reads it.
    let limit = input_data.max_amount.filter(|max_amount| *max_amount > 0);

    let result = crate::db_operations::read::get_highest_row_and_below(
        &namespace,
        &db_table,
        input_data.partition_key.as_str(),
        &input_data.row_key,
        limit,
        input_data.get_update_statistics(),
    );

    Ok(result.into())
}
