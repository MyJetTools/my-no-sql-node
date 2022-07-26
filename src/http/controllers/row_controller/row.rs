use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};
use my_http_server_controllers::controllers::actions::GetAction;
use my_http_server_controllers::controllers::documentation::out_results::HttpResult;
use my_http_server_controllers::controllers::documentation::HttpActionDescription;
use my_no_sql_core::db::UpdateExpirationTimeModel;

use crate::app::AppContext;

use super::models::{BaseDbRowContract, GetRowInputModel};

pub struct RowAction {
    app: Arc<AppContext>,
}

impl RowAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl GetAction for RowAction {
    fn get_route(&self) -> &str {
        "/Row"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: super::consts::CONTROLLER_NAME,
            description: "Get Entities",

            input_params: GetRowInputModel::get_input_params().into(),
            results: vec![
                HttpResult {
                    http_code: 200,
                    nullable: false,
                    description: "Rows".to_string(),
                    data_type: BaseDbRowContract::get_http_data_structure()
                        .into_http_data_type_array(),
                },
                crate::http::docs::rejects::op_with_table_is_failed(),
            ],
        }
        .into()
    }

    async fn handle_request(&self, ctx: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let input_data = GetRowInputModel::parse_http_input(ctx).await?;

        let db_table = crate::db_operations::read::table::get(
            self.app.as_ref(),
            input_data.table_name.as_ref(),
        )
        .await?;

        let update_expiration = UpdateExpirationTimeModel::new(
            input_data.set_db_rows_expiration_time.as_ref(),
            input_data.set_partition_expiration_time.as_ref(),
        );

        if let Some(partition_key) = input_data.partition_key.as_ref() {
            if let Some(row_key) = input_data.row_key.as_ref() {
                let result = crate::db_operations::read::rows::get_single(
                    self.app.as_ref(),
                    db_table.as_ref(),
                    partition_key,
                    row_key,
                    update_expiration,
                )
                .await?;

                return Ok(result.into());
            } else {
                let result = crate::db_operations::read::rows::get_all_by_partition_key(
                    self.app.as_ref(),
                    db_table.as_ref(),
                    partition_key,
                    input_data.limit,
                    input_data.skip,
                    update_expiration,
                )
                .await?;

                return Ok(result.into());
            }
        } else {
            if let Some(row_key) = input_data.row_key.as_ref() {
                let result = crate::db_operations::read::rows::get_all_by_row_key(
                    self.app.as_ref(),
                    db_table.as_ref(),
                    row_key,
                    input_data.limit,
                    input_data.skip,
                    update_expiration,
                )
                .await?;

                return Ok(result.into());
            } else {
                let result = crate::db_operations::read::rows::get_all(
                    self.app.as_ref(),
                    db_table.as_ref(),
                    input_data.limit,
                    input_data.skip,
                    update_expiration,
                )
                .await?;

                return Ok(result.into());
            }
        }
    }
}
