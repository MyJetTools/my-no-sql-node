use my_http_server::{HttpOkResult, HttpOutput, WebContentType};

use crate::db_operations::read::ReadOperationResult;

impl From<ReadOperationResult> for HttpOkResult {
    fn from(src: ReadOperationResult) -> Self {
        let content = match src {
            ReadOperationResult::SingleRow(content) => content,
            ReadOperationResult::RowsArray(content) => content,
            ReadOperationResult::EmptyArray => {
                vec![my_json::consts::OPEN_ARRAY, my_json::consts::CLOSE_ARRAY]
            }
        };

        HttpOkResult {
            write_telemetry: true,
            output: HttpOutput::Content {
                status_code: 200,
                headers: WebContentType::Json.into(),
                content,
            },
        }
    }
}
