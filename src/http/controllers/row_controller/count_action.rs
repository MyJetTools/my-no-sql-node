use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use crate::app::AppContext;

use super::models::RowsCountInputContract;

#[my_http_server_swagger::http_route(
    method: "GET",
    route: "/Count",
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
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let db_table =
        crate::db_operations::read::table::get(action.app.as_ref(), input_data.table_name.as_str())
            .await?;

    if let Some(partition_key) = input_data.partition_key {
        let table_access = db_table.data.read().await;

        let partition = table_access.get_partition(partition_key.as_str());

        if let Some(partition) = partition {
            return HttpOutput::as_text(partition.rows_count().to_string())
                .into_ok_result(true)
                .into();
        } else {
            return HttpOutput::as_text("0".to_string())
                .into_ok_result(true)
                .into();
        }
    }

    let table_access = db_table.data.read().await;

    let mut result = 0;

    for partition in table_access.get_partitions() {
        result += partition.rows_count();
    }

    return HttpOutput::as_text(result.to_string())
        .into_ok_result(true)
        .into();
}
