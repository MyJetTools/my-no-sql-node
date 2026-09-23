use my_http_server::{HttpFailResult, HttpOutput, WebContentType};

use crate::db_operations::DbOperationError;

use super::{OperationFailHttpContract, OperationFailReason};

/// Status code of an operation failure described by `OperationFailHttpContract` - the one the
/// main node answers with, and the one the SDK parses the contract of.
const OPERATION_FAIL_HTTP_STATUS_CODE: u16 = 400;

impl From<DbOperationError> for HttpFailResult {
    fn from(src: DbOperationError) -> Self {
        match src {
            DbOperationError::TableNotFound(table_name) => operation_fail(
                OperationFailReason::TableNotFound,
                format!("Table '{}' not found", table_name),
            ),
            DbOperationError::NamespaceNotFound(namespace) => operation_fail(
                OperationFailReason::NamespaceNotFound,
                format!("Namespace '{}' not found", namespace),
            ),
            DbOperationError::NamespaceNameValidationError(reason) => operation_fail(
                OperationFailReason::RequiredEntityFieldIsMissing,
                format!("Invalid namespace name: {}", reason),
            ),
            DbOperationError::RecordNotFound => HttpOutput::Content {
                headers: WebContentType::Text.into(),
                status_code: 404,
                content: b"Record not found".to_vec(),
            }
            .into_http_fail_result(false, false),
        }
    }
}

fn operation_fail(reason: OperationFailReason, message: String) -> HttpFailResult {
    let content = serde_json::to_vec(&OperationFailHttpContract { reason, message }).unwrap();

    HttpOutput::Content {
        headers: WebContentType::Json.into(),
        status_code: OPERATION_FAIL_HTTP_STATUS_CODE,
        content,
    }
    .into_http_fail_result(false, false)
}
