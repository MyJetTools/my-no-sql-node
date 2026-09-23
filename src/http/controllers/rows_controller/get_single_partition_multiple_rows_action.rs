use my_http_server::macros::*;
use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};

use crate::app::AppContext;

use super::models::GetSinglePartitionMultipleRowsActionInputContract;

#[http_route(
    method: "POST",
    route: "/api/Rows/SinglePartitionMultipleRows",
    deprecated_routes: ["/Rows/SinglePartitionMultipleRows"],
    controller: "Rows",
    description: "Gets the rows of a partition by their row keys",
    summary: "Returns the rows of a partition by their row keys",
    input_data: "GetSinglePartitionMultipleRowsActionInputContract",
    result:[
        {status_code: 200, description: "Rows"},
    ]
)]
pub struct GetSinglePartitionMultipleRowsAction {
    app: Arc<AppContext>,
}

impl GetSinglePartitionMultipleRowsAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &GetSinglePartitionMultipleRowsAction,
    input_data: GetSinglePartitionMultipleRowsActionInputContract,
    ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let namespace = crate::http::get_request_namespace(&action.app, ctx)?;

    let db_table =
        crate::db_operations::read::get_table(&namespace, input_data.table_name.as_str())?;

    // A body which is not an array of row keys is a 400, not a panic.
    let row_keys = input_data.body.deserialize_json()?;

    let result = crate::db_operations::read::rows::get_single_partition_multiple_rows(
        &namespace,
        &db_table,
        input_data.partition_key.as_str(),
        row_keys.as_slice(),
        input_data.get_update_statistics(),
    );

    Ok(result.into())
}
