use std::sync::Arc;

use my_http_server::{HttpFailResult, WebContentType};
use my_no_sql_core::db::DbTable;

use crate::{app::AppContext, http::mappers::db_operation_error::OPERATION_FAIL_HTTP_STATUS_CODE};

use super::mappers::{OperationFailHttpContract, OperationFailReason};

#[async_trait::async_trait]
pub trait GetTableHttpSupport {
    async fn get_table(&self, table_name: &str) -> Result<Arc<DbTable>, HttpFailResult>;
}
pub fn table_not_found_http_result(table_name: &str) -> HttpFailResult {
    let err_model = OperationFailHttpContract {
        reason: OperationFailReason::TableNotFound,
        message: format!("Table '{}' not found", table_name),
    };
    let content = serde_json::to_vec(&err_model).unwrap();

    HttpFailResult {
        content_type: WebContentType::Json,
        status_code: OPERATION_FAIL_HTTP_STATUS_CODE,
        content,
        write_telemetry: true,
    }
}

#[async_trait::async_trait]
impl GetTableHttpSupport for AppContext {
    async fn get_table(&self, table_name: &str) -> Result<Arc<DbTable>, HttpFailResult> {
        if let Some(db_table) = self.db.get_table(table_name).await {
            return Ok(db_table);
        }

        Err(table_not_found_http_result(table_name))
    }
}
