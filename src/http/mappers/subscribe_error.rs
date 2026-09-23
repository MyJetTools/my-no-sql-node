use my_http_server::{HttpFailResult, HttpOutput, WebContentType};

use crate::operations::SubscribeError;

impl From<SubscribeError> for HttpFailResult {
    fn from(src: SubscribeError) -> Self {
        HttpOutput::Content {
            headers: WebContentType::Text.into(),
            status_code: 400,
            content: src.to_string().into_bytes(),
        }
        .into_http_fail_result(false, false)
    }
}
