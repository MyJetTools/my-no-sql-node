use my_http_server::macros::*;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use std::sync::Arc;

use crate::app::AppContext;

use super::models::RowsCountInputContract;

#[http_route(
    method: "GET",
    route: "/api/Count",
    deprecated_routes: ["/Count"],
    controller: "Row",
    description: "Get Rows Count",
    summary: "Returns Rows Count",
    input_data: "RowsCountInputContract",
    result:[
        {status_code: 200, description: "Amount of rows of the table or the partition"},
    ]
)]
pub struct RowCountAction {
    app: Arc<AppContext>,
}

impl RowCountAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &RowCountAction,
    input_data: RowsCountInputContract,
    ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let namespace = crate::http::get_request_namespace(&action.app, ctx)?;

    let db_table =
        crate::db_operations::read::get_table(&namespace, input_data.table_name.as_str())?;

    let rows_count = {
        let table_access = db_table.data.read();

        match input_data.partition_key.as_ref() {
            Some(partition_key) => table_access
                .get_partition(partition_key.as_str())
                .map(|partition| partition.rows_count())
                .unwrap_or(0),
            None => table_access.get_rows_amount(),
        }
    };

    HttpOutput::as_text(rows_count.to_string()).into_ok_result(true)
}
