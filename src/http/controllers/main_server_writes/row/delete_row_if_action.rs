use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};

use crate::app::AppContext;
use super::super::models::*;

#[http_route(
    method: "DELETE",
    route: "/api/Row/DeleteIf",
    controller: "Row",
    description: "Deletes the row only when it is still at the given version",
    summary: "Deletes the row only when the TimeStamp stored in the table is still the one sent here. A row which has been rewritten meanwhile answers 409 and stays in place",
    input_data: "DeleteRowIfInputModel",
    result:[
        {status_code: 200, description: "Deleted row",  model:"BaseDbRowContract"},
        {status_code: 400, description: "Table not found, or timeStamp is not a valid TimeStamp"},
        {status_code: 404, description: "Row not found"},
        {status_code: 409, description: "Row has been changed since it was read"},
    ],
)]
pub struct DeleteRowIfAction {
    app: Arc<AppContext>,
}

impl DeleteRowIfAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

/// Forwarded to the main node - validated first the way the main node would.
async fn handle_request(
    action: &DeleteRowIfAction,
    _input_data: DeleteRowIfInputModel,
    ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    crate::main_server_http::forward(&action.app, ctx).await
}
