use my_http_server::macros::*;
use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};

use crate::app::AppContext;

use rest_api_shared::GetRowInputModel;

#[http_route(
    method: "GET",
    route: "/api/Row",
    deprecated_routes: ["/Row"],
    controller: "Row",
    description: "Get Entity or entities",
    summary: "Returns Entity or entities",
    input_data: "GetRowInputModel",
    result:[
        {status_code: 200, description: "Single Row or array of rows"},
    ]
)]
pub struct GetRowsAction {
    app: Arc<AppContext>,
}

impl GetRowsAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &GetRowsAction,
    input_data: GetRowInputModel,
    ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let namespace = crate::http::get_request_namespace(&action.app, ctx)?;

    let db_table =
        crate::db_operations::read::get_table(&namespace, input_data.table_name.as_str())?;

    let update_statistics = super::models::get_update_statistics(&input_data);

    let result = match (
        input_data.partition_key.as_ref(),
        input_data.row_key.as_ref(),
    ) {
        (Some(partition_key), Some(row_key)) => crate::db_operations::read::rows::get_single(
            &namespace,
            &db_table,
            partition_key,
            row_key,
            update_statistics,
        )?,
        (Some(partition_key), None) => crate::db_operations::read::rows::get_all_by_partition_key(
            &namespace,
            &db_table,
            partition_key,
            input_data.limit,
            input_data.skip,
            update_statistics,
        ),
        (None, Some(row_key)) => crate::db_operations::read::rows::get_all_by_row_key(
            &namespace,
            &db_table,
            row_key,
            input_data.limit,
            input_data.skip,
            update_statistics,
        ),
        (None, None) => crate::db_operations::read::rows::get_all(
            &namespace,
            &db_table,
            input_data.limit,
            input_data.skip,
            update_statistics,
        ),
    };

    Ok(result.into())
}
