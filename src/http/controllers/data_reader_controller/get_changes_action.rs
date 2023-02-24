use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput, WebContentType};

use crate::{
    app::AppContext,
    data_readers::{http_connection::HttpPayload, DataReaderConnection},
    db_operations::DbOperationError,
    http::{controllers::mappers::ToSetExpirationTime, http_sessions::HttpSessionsSupport},
};

use super::models::{GetChangesInputModel, UpdateExpirationDateTime};

#[my_http_server_swagger::http_route(
    method: "POST",
    route: "/DataReader/GetChanges",
    controller: "DataReader",
    description: "Get Subscriber changes",
    summary: "Returns Subscriber changes",
    input_data: "GetChangesInputModel",
    result:[
        {status_code: 200, description: "Successful operation"},
    ]
)]
pub struct GetChangesAction {
    app: Arc<AppContext>,
}

impl GetChangesAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &GetChangesAction,
    input_data: GetChangesInputModel,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let data_reader = action
        .app
        .get_http_session(input_data.session_id.as_str())
        .await?;

    let body_data = input_data.body.deserialize_json()?;
    for update_model in &body_data.update_expiration_time {
        update_expiration_time(
            action.app.as_ref(),
            update_model.table_name.as_str(),
            &update_model.items,
        )
        .await?;
    }

    if let DataReaderConnection::Http(info) = &data_reader.connection {
        let result = info.new_request().await?;
        match result {
            HttpPayload::Ping => return HttpOutput::Empty.into_ok_result(false).into(),
            HttpPayload::Payload(payload) => {
                return HttpOutput::Content {
                    headers: None,
                    content_type: None,
                    content: payload,
                }
                .into_ok_result(false)
                .into();
            }
        }
    }

    return Err(HttpFailResult {
        content_type: WebContentType::Text,
        status_code: 400,
        content: "Only HTTP sessions are supported".to_string().into_bytes(),
        write_telemetry: true,
        write_to_log: true,
    });
}

async fn update_expiration_time(
    app: &AppContext,
    table_name: &str,
    items: &[UpdateExpirationDateTime],
) -> Result<(), DbOperationError> {
    let db_table = app.db.get_table(table_name).await;
    if db_table.is_none() {
        return Ok(());
    }

    for item in items {
        if let Some(set_expiration_time) = item
            .set_db_partition_expiration_time
            .to_set_expiration_time()
        {
            app.sync_to_main_node
                .event_notifier
                .update_partition_expiration_time(
                    table_name,
                    &item.partition_key,
                    set_expiration_time,
                )
                .await;
        }

        if let Some(set_expiration_time) = item.set_db_rows_expiration_time.to_set_expiration_time()
        {
            app.sync_to_main_node
                .event_notifier
                .update_rows_expiration_time(
                    table_name,
                    &item.partition_key,
                    item.row_keys.iter(),
                    set_expiration_time,
                )
                .await;
        }
    }

    Ok(())
}
