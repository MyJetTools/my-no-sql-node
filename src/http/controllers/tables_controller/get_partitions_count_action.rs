use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use super::models::GetPartitionsAmountContract;
use crate::app::AppContext;
use std::{result::Result, sync::Arc};

#[my_http_server_swagger::http_route(
    method: "GET",
    route: "/Tables/PartitionsCount",
    input_data: "GetPartitionsAmountContract",
    description: "Get Partitions amount of selected table",
    controller: "Tables",
    result:[
        {status_code: 200, description: "Partitions amount", model: "Long"},
        {status_code: 400, description: "Table not found"},
    ]
)]
pub struct GetPartitionsCountAction {
    app: Arc<AppContext>,
}

impl GetPartitionsCountAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &GetPartitionsCountAction,
    input_data: GetPartitionsAmountContract,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let db_table =
        crate::db_operations::read::table::get(action.app.as_ref(), input_data.table_name.as_str())
            .await?;

    let partitions_amount = db_table.get_partitions_amount().await;

    HttpOutput::as_text(format!("{}", partitions_amount))
        .into_ok_result(true)
        .into()
}

/*
#[async_trait]
impl GetAction for GetPartitionsCountAction {
    fn get_route(&self) -> &str {
        "/Tables/PartitionsCount"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: super::consts::CONTROLLER_NAME,
            description: "Get Partitions count",

            input_params: GetPartitionsAmountContract::get_input_params().into(),
            results: vec![
                HttpResult {
                    http_code: 200,
                    nullable: true,
                    description: "Partitions count".to_string(),
                    data_type: HttpDataType::as_long(),
                },
                response::table_not_found(),
            ],
        }
        .into()
    }


}
 */
