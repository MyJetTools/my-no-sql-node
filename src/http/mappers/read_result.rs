use my_http_server::{HttpOkResult, HttpOutput, WebContentType};

use crate::db_operations::read::ReadOperationResult;

impl Into<HttpOkResult> for ReadOperationResult {
    fn into(self) -> HttpOkResult {
        match self {
            ReadOperationResult::SingleRow(content) => {
                let output = HttpOutput::Content {
                    headers: None,
                    content,
                    content_type: Some(WebContentType::Json),
                };

                HttpOkResult {
                    write_telemetry: true,
                    output,
                }
            }
            ReadOperationResult::RowsArray(content) => {
                let output = HttpOutput::Content {
                    headers: None,
                    content,
                    content_type: Some(WebContentType::Json),
                };

                HttpOkResult {
                    write_telemetry: true,
                    output,
                }
            }
            ReadOperationResult::EmptyArray => {
                let empty_array = vec![
                    my_json::json_reader::consts::OPEN_ARRAY,
                    my_json::json_reader::consts::CLOSE_ARRAY,
                ];

                let output = HttpOutput::Content {
                    headers: None,
                    content: empty_array,
                    content_type: Some(WebContentType::Json),
                };

                HttpOkResult {
                    write_telemetry: true,
                    output,
                }
            }
        }
    }
}
