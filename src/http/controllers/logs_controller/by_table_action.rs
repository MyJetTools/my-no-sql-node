use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput, WebContentType};
use my_http_server_controllers::controllers::{
    actions::GetAction, documentation::HttpActionDescription,
};
use rust_extensions::StopWatch;

use crate::app::AppContext;

pub struct LogsByTableAction {
    app: Arc<AppContext>,
}

impl LogsByTableAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl GetAction for LogsByTableAction {
    fn get_route(&self) -> &str {
        "/Logs/Table/{table_name}"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        None
    }

    async fn handle_request(&self, ctx: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let table_name = get_table_name(ctx.request.get_path());

        if table_name.is_none() {
            return render_select_table(self.app.as_ref()).await.into();
        }

        let table_name = table_name.unwrap();

        let mut sw = StopWatch::new();
        sw.start();
        let logs_result = self.app.logs.get_by_table_name(table_name).await;

        match logs_result {
            Some(logs) => super::logs::compile_result("logs by table", logs, sw).into(),
            None => {
                sw.pause();

                let content = format!(
                    "Result compiled in: {:?}. No log recods for the table '{}'",
                    sw.duration(),
                    table_name
                );

                HttpOutput::Content {
                    headers: None,
                    content_type: Some(WebContentType::Text),
                    content: content.into_bytes(),
                }
                .into_ok_result(true)
                .into()
            }
        }
    }
}

fn get_table_name(path: &str) -> Option<&str> {
    let segments = path.split('/');

    let mut value = "";
    let mut amount: usize = 0;
    for segment in segments {
        value = segment;
        amount += 1;
    }

    if amount == 4 {
        return Some(value);
    }

    None
}

async fn render_select_table(app: &AppContext) -> HttpOkResult {
    let mut body = String::new();

    body.push_str("<h1>Please, select table to show logs</h1>");

    for table_name in &app.db.get_table_names().await {
        let line = format!(
            "<a class='btn btn-sm btn-outline-primary' href='/logs/table/{table_name}'>{table_name}</a>",
        );
        body.push_str(line.as_str());
    }

    super::super::as_html::build("Select table to show logs", body.as_str())
        .into_ok_result(true)
        .into()
}
