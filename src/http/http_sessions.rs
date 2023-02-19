use std::sync::Arc;

use my_http_server::{HttpFailResult, WebContentType};

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext,
    data_readers::{DataReader, DataReaderConnection},
};

#[async_trait::async_trait]
pub trait HttpSessionsSupport {
    async fn get_http_session(&self, session_id: &str) -> Result<Arc<DataReader>, HttpFailResult>;
}

#[async_trait::async_trait]
impl HttpSessionsSupport for AppContext {
    async fn get_http_session(&self, session_id: &str) -> Result<Arc<DataReader>, HttpFailResult> {
        if let Some(result) = self.data_readers.get_http(session_id).await {
            if let DataReaderConnection::Http(info) = &result.connection {
                info.last_incoming_moment
                    .update(DateTimeAsMicroseconds::now());
            }
            return Ok(result);
        }

        let err = HttpFailResult {
            content_type: WebContentType::Text,
            status_code: SESSION_NOT_FOUND_HTTP_CODE,
            content: "Session not found".to_string().into_bytes(),
            write_telemetry: false,
            write_to_log: true,
        };

        Err(err)
    }
}

const SESSION_NOT_FOUND_HTTP_CODE: u16 = 403;
