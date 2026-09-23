use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};

use crate::app::AppContext;
use super::super::models::*;

#[http_route(
    method: "POST",
    route: "/api/Transactions/Cancel",
    deprecated_routes: ["/Transactions/Cancel"],
    description: "Cancel transaction",
    summary: "Cancels transaction",
    input_data: "ProcessTransactionInputModel",
    controller: "Transactions",
    result:[
        {status_code: 202, description: "Transaction is canceled"},        
    ],
)]
pub struct CancelTransactionAction {
    app: Arc<AppContext>,
}

impl CancelTransactionAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

/// Forwarded to the main node - validated first the way the main node would.
async fn handle_request(
    action: &CancelTransactionAction,
    _input_data: ProcessTransactionInputModel,
    ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    crate::main_server_http::forward(&action.app, ctx).await
}
