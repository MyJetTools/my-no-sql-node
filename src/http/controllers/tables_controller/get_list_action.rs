use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use crate::app::AppContext;

use super::models::TableContract;

#[my_http_server_swagger::http_route(
    method: "GET",
    route: "/Tables/List",
    description: "Get List of Tables",
    controller: "Tables",
    result:[
        {status_code: 200, description: "List of tables", model_as_array: "TableContract"},
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
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    crate::db_operations::check_app_states(action.app.as_ref())?;
    let tables = action.app.db.get_tables().await;

    let mut response: Vec<TableContract> = vec![];

    for db_table in &tables {
        response.push(db_table.as_ref().into());
    }

    HttpOutput::as_json(response).into_ok_result(true).into()
}
