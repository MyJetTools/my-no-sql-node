use crate::app::AppContext;
use my_http_server::macros::*;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use rest_api_shared::{
    GetPartitionsDetailsContract, PartitionDetailsContract, PartitionsDetailsContract,
};
use std::{result::Result, sync::Arc};

#[http_route(
    method: "GET",
    route: "/api/Partitions/Details",
    input_data: "GetPartitionsDetailsContract",
    description: "Get per-partition metrics of selected table",
    summary: "Returns records count and data size in bytes of the partitions of the table matching the filter - 1000 of them at most",
    controller: "Partitions",
    result:[
        {status_code: 200, description: "Per-partition metrics", model: "PartitionsDetailsContract"},
        {status_code: 400, description: "Table not found"},
    ]
)]
pub struct GetPartitionsDetailsAction {
    app: Arc<AppContext>,
}

impl GetPartitionsDetailsAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &GetPartitionsDetailsAction,
    input_data: GetPartitionsDetailsContract,
    ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let namespace = crate::http::get_request_namespace(&action.app, ctx)?;

    let db_table =
        crate::db_operations::read::get_table(&namespace, input_data.table_name.as_str())?;

    let details = crate::db_operations::read::partitions::get_partitions_details(
        &db_table,
        input_data.filter.as_deref(),
        input_data.limit,
    );

    let result = PartitionsDetailsContract {
        total: details.total as u64,
        matched: details.matched as u64,
        partitions: details
            .partitions
            .into_iter()
            .map(|itm| PartitionDetailsContract {
                partition_key: itm.partition_key,
                records_count: itm.records_count as u64,
                data_size: itm.data_size as u64,
            })
            .collect(),
    };

    HttpOutput::as_json(result).into_ok_result(true)
}
