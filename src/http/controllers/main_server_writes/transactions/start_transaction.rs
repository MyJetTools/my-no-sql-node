use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};

use crate::app::AppContext;
use super::super::models::*;

#[http_route(
    method: "POST",
    route: "/api/Transactions/Start",
    deprecated_routes: ["/Transactions/Start"],
    description: "Start new Transaction",
    summary: "Starts new Transaction",
    controller: "Transactions",
    result:[
        {status_code: 200, description: "Issued transaction", model: "StartTransactionResponse"},        
    ],
)]
pub struct StartTransactionAction {
    app: Arc<AppContext>,
}

impl StartTransactionAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

/// Forwarded to the main node - validated first the way the main node would.
async fn handle_request(
    action: &StartTransactionAction,
    ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    crate::main_server_http::forward(&action.app, ctx).await
}
