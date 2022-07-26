use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};
use my_http_server_controllers::controllers::{
    actions::GetAction,
    documentation::{out_results::HttpResult, HttpActionDescription},
};
use my_no_sql_core::db::UpdateExpirationTimeModel;

use crate::{app::AppContext, http::controllers::row_controller::models::BaseDbRowContract};

use super::models::GetHighestRowsAndBelowInputContract;

pub struct GetHighestRowAndBelowAction {
    app: Arc<AppContext>,
}

impl GetHighestRowAndBelowAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl GetAction for GetHighestRowAndBelowAction {
    fn get_route(&self) -> &str {
        "/Rows/HighestRowAndBelow"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: super::consts::CONTROLLER_NAME,
            description: "Get Rows Count",

            input_params: GetHighestRowsAndBelowInputContract::get_input_params().into(),
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
        let input_data = GetHighestRowsAndBelowInputContract::parse_http_input(ctx).await?;

        let db_table = crate::db_operations::read::table::get(
            self.app.as_ref(),
            input_data.table_name.as_ref(),
        )
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

        let update_expiration = UpdateExpirationTimeModel::new(
            input_data.set_db_rows_expiration_time.as_ref(),
            input_data.set_partition_expiration_time.as_ref(),
        );

        let result = crate::db_operations::read::get_highest_row_and_below(
            self.app.as_ref(),
            db_table.as_ref(),
            input_data.partition_key.as_ref(),
            &input_data.row_key,
            limit,
            update_expiration,
        )
        .await?;

        Ok(result.into())
    }
}
