use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};

use crate::app::AppContext;
use super::super::models::*;

#[http_route(
    method: "POST",
    route: "/api/Transactions/Commit",
    deprecated_routes: ["/Transactions/Commit"],
    description: "Commit transaction",
    summary: "Commits transaction",
    input_data: "ProcessTransactionInputModel",
    controller: "Transactions",
    result:[
        {status_code: 202, description: "Transaction is canceled"},        
    ],
)]
pub struct CommitTransactionAction {
    app: Arc<AppContext>,
}

impl CommitTransactionAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

/// Forwarded to the main node - validated first the way the main node would.
async fn handle_request(
    action: &CommitTransactionAction,
    _input_data: ProcessTransactionInputModel,
    ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    crate::main_server_http::forward(&action.app, ctx).await
}
