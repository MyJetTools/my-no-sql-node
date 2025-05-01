use my_http_server::{HttpFailResult, WebContentType};

use crate::http::mappers::db_operation_error::OPERATION_FAIL_HTTP_STATUS_CODE;

use super::mappers::{OperationFailHttpContract, OperationFailReason};

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
        write_to_log: true,
    }
}
