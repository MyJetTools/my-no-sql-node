use std::sync::Arc;

use my_http_server::{HttpFailResult, HttpOutput, WebContentType};
use my_no_sql_sdk::core::rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext,
    data_readers::{DataReader, DataReaderConnection},
};

const SESSION_NOT_FOUND_HTTP_CODE: u16 = 403;

pub fn get_http_session(
    app: &AppContext,
    session_id: &str,
) -> Result<Arc<DataReader>, HttpFailResult> {
    let Some(data_reader) = app.data_readers.get_http(session_id) else {
        return Err(session_not_found());
    };

    if let DataReaderConnection::Http(info) = &data_reader.connection {
        info.last_incoming_moment
            .update(DateTimeAsMicroseconds::now());
    }

    Ok(data_reader)
}

pub fn session_not_found() -> HttpFailResult {
    HttpOutput::Content {
        headers: WebContentType::Text.into(),
        status_code: SESSION_NOT_FOUND_HTTP_CODE,
        content: b"Session not found".to_vec(),
    }
    .into_http_fail_result(false, false)
}
