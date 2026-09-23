use my_http_server::macros::*;
use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};

use crate::app::AppContext;

use super::models::GetHighestRowsAndBelowInputContract;

#[http_route(
    method: "GET",
    route: "/Rows/HighestRowAndBelow",
    controller: "Rows",
    description: "Gets row with highest row_key and below",
    summary: "Returns row with highest row_key and below",
    input_data: "GetHighestRowsAndBelowInputContract",
    result:[
        {status_code: 200, description: "Monitoring snapshot"},
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
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let db_table =
        crate::db_operations::read::table::get(action.app.as_ref(), input_data.table_name.as_ref())
            .await?;

    let limit = if let Some(max_amount) = input_data.max_amount {
        if max_amount == 0 {
            None
        } else {
            Some(max_amount)
        }
    } else {
        None
    };

    let result = crate::db_operations::read::get_highest_row_and_below(
        &action.app,
        db_table.as_ref(),
        &input_data.partition_key,
        &input_data.row_key,
        limit,
        input_data.get_update_statistics(),
    )
    .await?;

    Ok(result.into())
}
