use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput, WebContentType};
use my_http_server_controllers::controllers::{
    actions::PostAction,
    documentation::{data_types::HttpDataType, out_results::HttpResult, HttpActionDescription},
};

use crate::{
    app::AppContext,
    data_readers::{http_connection::HttpPayload, DataReaderConnection},
    db_operations::DbOperationError,
    http::http_sessions::HttpSessionsSupport,
};

use super::models::{GetChangesBodyModel, GetChangesInputModel, UpdateExpirationDateTime};

pub struct GetChangesAction {
    app: Arc<AppContext>,
}

impl GetChangesAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}
#[async_trait::async_trait]
impl PostAction for GetChangesAction {
    fn get_route(&self) -> &str {
        "/DataReader/GetChanges"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: super::consts::CONTROLLER_NAME,
            description: "Get Subscriber changes",

            input_params: GetChangesInputModel::get_input_params().into(),
            results: vec![HttpResult {
                http_code: 200,
                nullable: true,
                description: "Successful operation".to_string(),
                data_type: HttpDataType::None,
            }],
        }
        .into()
    }

    async fn handle_request(&self, ctx: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let input_data = GetChangesInputModel::parse_http_input(ctx).await?;

        let data_reader = self
            .app
            .get_http_session(input_data.session_id.as_str())
            .await?;

        if let Ok(body) = serde_json::from_slice::<GetChangesBodyModel>(&input_data.body) {
            for update_model in &body.update_expiration_time {
                update_expiration_time(
                    self.app.as_ref(),
                    update_model.table_name.as_str(),
                    &update_model.items,
                )
                .await?;
            }
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
        });
    }
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

    let db_table = db_table.unwrap();
    for item in items {
        //    let src = EventSource::as_client_request(app);

        //let update_expiration = UpdateExpirationTimeModel::new(
        //    item.set_db_rows_expiration_time.as_ref(),
        //    item.set_db_partition_expiration_time.as_ref(),
        //);

        //if let Some(update_expiration) = &update_expiration {
        //TODO - UpdateExpirationTime to MasterNode
        //}
    }

    Ok(())
}
