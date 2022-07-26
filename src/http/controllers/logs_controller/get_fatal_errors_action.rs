use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput, WebContentType};
use my_http_server_controllers::controllers::{
    actions::GetAction, documentation::HttpActionDescription,
};
use rust_extensions::StopWatch;

use crate::app::AppContext;

pub struct GetFatalErrorsAction {
    app: Arc<AppContext>,
}

impl GetFatalErrorsAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl GetAction for GetFatalErrorsAction {
    fn get_route(&self) -> &str {
        "/Logs/FatalErrors"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        None
    }

    async fn handle_request(&self, _ctx: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let mut sw = StopWatch::new();
        sw.start();
        let logs_result = self.app.logs.get_fatal_errors().await;

        match logs_result {
            Some(logs) => super::logs::compile_result("FatalError logs", logs, sw).into(),
            None => {
                sw.pause();

                let content = format!(
                    "Result compiled in: {:?}. No fatal error records",
                    sw.duration(),
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
