use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};

use crate::app::AppContext;

use super::models::GetRowInputModel;

#[my_http_server_swagger::http_route(
    method: "GET",
    route: "/Row",
    controller: "Row",
    description: "Get Entitity or entities",
    summary: "Returns Entitity or entities",
    input_data: "GetRowInputModel",
    result:[
        {status_code: 200, description: "Single Row or array of rows"},
    ]
)]
pub struct RowAction {
    app: Arc<AppContext>,
}

impl RowAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &RowAction,
    input_data: GetRowInputModel,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let db_table_wrapper =
        crate::db_operations::read::table::get(action.app.as_ref(), input_data.table_name.as_ref())
            .await?;

    if let Some(partition_key) = input_data.partition_key.as_ref() {
        if let Some(row_key) = input_data.row_key.as_ref() {
            let result = crate::db_operations::read::rows::get_single(
                &action.app,
                db_table_wrapper.as_ref(),
                partition_key,
                row_key,
                input_data.get_update_statistics(),
            )
            .await?;

            return Ok(result.into());
        } else {
            let result = crate::db_operations::read::rows::get_all_by_partition_key(
                &action.app,
                db_table_wrapper.as_ref(),
                partition_key,
                input_data.limit,
                input_data.skip,
                input_data.get_update_statistics(),
            )
            .await?;

            return Ok(result.into());
        }
    } else {
        if let Some(row_key) = input_data.row_key.as_ref() {
            let result = crate::db_operations::read::rows::get_all_by_row_key(
                &action.app,
                db_table_wrapper.as_ref(),
                row_key,
                input_data.limit,
                input_data.skip,
                input_data.get_update_statistics(),
            )
            .await?;

            return Ok(result.into());
        } else {
            let result = crate::db_operations::read::rows::get_all(
                &action.app,
                db_table_wrapper.as_ref(),
                input_data.limit,
                input_data.skip,
                input_data.get_update_statistics(),
            )
            .await?;

            return Ok(result.into());
        }
    }
}
