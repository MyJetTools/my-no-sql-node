use my_http_server::macros::*;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use std::sync::Arc;

use crate::app::AppContext;

use super::models::{GetTablesListContract, TableContract};

#[http_route(
    method: "GET",
    route: "/api/Tables/List",
    deprecated_routes: ["/Tables/List"],
    description: "Get List of Tables",
    summary: "Returns the tables of the namespace this node replicates",
    controller: "Tables",
    input_data: "GetTablesListContract",
    result:[
        {status_code: 200, description: "List of tables", model: "Vec<TableContract>"},
    ]
)]
pub struct GetListAction {
    app: Arc<AppContext>,
}

impl GetListAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &GetListAction,
    _input_data: GetTablesListContract,
    ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let namespace = crate::http::get_request_namespace(&action.app, ctx)?;

    let response: Vec<TableContract> = namespace
        .db
        .get_tables()
        .iter()
        .map(|db_table| TableContract {
            name: db_table.name.to_string(),
        })
        .collect();

    HttpOutput::as_json(response).into_ok_result(true)
}
