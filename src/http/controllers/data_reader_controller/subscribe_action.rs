use my_http_server::macros::*;
use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use crate::{app::AppContext, db_operations::DbOperationError};

use super::models::SubscribeToTableInputModel;

#[http_route(
    method: "POST",
    route: "/api/DataReader/Subscribe",
    deprecated_routes: ["/DataReader/Subscribe"],
    controller: "DataReader",
    summary: "Subscribes to table",
    description: "Subscribe to table",
    input_data: "SubscribeToTableInputModel",
    result:[
        {status_code: 202, description: "Successful operation"},
    ]
)]
pub struct SubscribeAction {
    app: Arc<AppContext>,
}

impl SubscribeAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &SubscribeAction,
    input_data: SubscribeToTableInputModel,
    ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let data_reader = crate::http::get_http_session(&action.app, input_data.session_id.as_str())?;

    // An HTTP reader names its namespace on the subscribe request - every subscription of the
    // session has to name the same one.
    let namespace =
        crate::http::parse_namespace_name(crate::http::get_request_namespace_name(ctx))?;

    if let Err(err) = data_reader.set_namespace(namespace.into()) {
        return Err(DbOperationError::NamespaceNameValidationError(err).into());
    }

    crate::operations::subscribe(&action.app, &data_reader, input_data.table_name.as_str()).await?;

    HttpOutput::Empty.into_ok_result(true)
}
