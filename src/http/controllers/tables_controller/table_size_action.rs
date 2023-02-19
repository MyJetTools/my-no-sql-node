use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use crate::app::AppContext;

use super::models::GetTableSizeContract;

#[my_http_server_swagger::http_route(
    method: "GET",
    route: "/Tables/TableSize",
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
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    crate::db_operations::check_app_states(action.app.as_ref())?;

    let db_table =
        crate::db_operations::read::table::get(action.app.as_ref(), input_data.table_name.as_str())
            .await?;

    let partitions_amount = db_table.get_table_size().await;

    HttpOutput::as_text(format!("{}", partitions_amount))
        .into_ok_result(true)
        .into()
}
