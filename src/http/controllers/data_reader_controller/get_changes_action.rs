use my_http_server::macros::*;
use my_no_sql_sdk::tcp_contracts::sync_to_main::UpdateEntityStatisticsData;
use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput, WebContentType};

use crate::{
    app::AppContext,
    data_readers::{DataReader, DataReaderConnection, HttpPayload},
};

use super::models::{GetChangesInputModel, UpdateExpirationDateTimeByTable};

#[http_route(
    method: "POST",
    route: "/api/DataReader/GetChanges",
    deprecated_routes: ["/DataReader/GetChanges"],
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
    let data_reader = crate::http::get_http_session(&action.app, input_data.session_id.as_str())?;

    let body_data = input_data.body.deserialize_json()?;

    for update_model in body_data.update_expiration_time.iter() {
        update_expiration_time(action.app.as_ref(), data_reader.as_ref(), update_model);
    }

    let DataReaderConnection::Http(info) = &data_reader.connection else {
        return HttpOutput::Content {
            status_code: 400,
            headers: WebContentType::Text.into(),
            content: b"Only HTTP sessions are supported".to_vec(),
        }
        .into_err(false, false);
    };

    match info.new_request().await {
        Some(HttpPayload::Ping) => HttpOutput::Empty.into_ok_result(false),
        Some(HttpPayload::Payload(payload)) => HttpOutput::Content {
            status_code: 200,
            headers: Default::default(),
            content: payload,
        }
        .into_ok_result(false),
        None => Err(crate::http::session_not_found()),
    }
}

fn update_expiration_time(
    app: &AppContext,
    data_reader: &DataReader,
    update_model: &UpdateExpirationDateTimeByTable,
) {
    let Some(namespace) = app.namespaces.get(data_reader.get_namespace().as_str()) else {
        return;
    };

    if namespace
        .db
        .get_table(update_model.table_name.as_str())
        .is_none()
    {
        return;
    }

    for item in update_model.items.iter() {
        if let Some(expiration_time) = item.get_db_partition_expiration_time() {
            namespace.main_node.sync_to_main_node.update(
                update_model.table_name.as_str(),
                item.partition_key.as_str(),
                || item.row_keys.iter().map(|itm| itm.as_str()),
                &UpdateEntityStatisticsData {
                    partition_expiration_moment: Some(Some(expiration_time)),
                    ..Default::default()
                },
            );
        }

        if let Some(expiration_time) = item.get_db_rows_expiration_time() {
            namespace.main_node.sync_to_main_node.update(
                update_model.table_name.as_str(),
                item.partition_key.as_str(),
                || item.row_keys.iter().map(|itm| itm.as_str()),
                &UpdateEntityStatisticsData {
                    row_expiration_moment: Some(Some(expiration_time)),
                    ..Default::default()
                },
            );
        }
    }
}
