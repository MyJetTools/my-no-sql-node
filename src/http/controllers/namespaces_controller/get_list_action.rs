use my_http_server::macros::*;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use std::sync::Arc;

use crate::app::AppContext;

use super::models::NamespaceContract;

#[http_route(
    method: "GET",
    route: "/api/Namespaces/List",
    controller: "Namespaces",
    description: "Get list of namespaces",
    summary: "Returns every namespace this node replicates, with the state of its connection to the main node",
    result:[
        {status_code: 200, description: "List of namespaces", model: "Vec<NamespaceContract>"},
    ]
)]
pub struct GetNamespacesListAction {
    app: Arc<AppContext>,
}

impl GetNamespacesListAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &GetNamespacesListAction,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let result: Vec<NamespaceContract> = action
        .app
        .namespaces
        .get_all()
        .iter()
        .map(|namespace| NamespaceContract {
            name: namespace.name.to_string(),
            tables_amount: namespace.db.get_tables().len(),
            connected_to_main_node: namespace.main_node.is_connected(),
            main_node_ping: namespace.main_node.get_ping_micros(),
        })
        .collect();

    HttpOutput::as_json(result).into_ok_result(false)
}
