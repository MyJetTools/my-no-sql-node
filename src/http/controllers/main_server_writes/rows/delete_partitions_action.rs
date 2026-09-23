use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};

use crate::app::AppContext;
use super::super::models::*;

#[http_route(
    method: "DELETE",
    route: "/api/Rows/DeletePartitions",
    deprecated_routes: ["/Rows/DeletePartitions"],
    controller: "Rows",
    description: "Delete Partitions",
    summary: "Deletes Partitions",
    input_data: "DeletePartitionsInputContract",
    result:[
        {status_code: 200, description: "Removed entities"},
    ],
)]
pub struct DeletePartitionsAction {
    app: Arc<AppContext>,
}

impl DeletePartitionsAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

/// Forwarded to the main node - validated first the way the main node would.
async fn handle_request(
    action: &DeletePartitionsAction,
    _input_data: DeletePartitionsInputContract,
    ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    crate::main_server_http::forward(&action.app, ctx).await
}
