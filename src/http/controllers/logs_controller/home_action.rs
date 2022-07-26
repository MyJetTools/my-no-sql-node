use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};
use my_http_server_controllers::controllers::{
    actions::GetAction, documentation::HttpActionDescription,
};
use rust_extensions::StopWatch;

use crate::app::AppContext;

pub struct HomeAction {
    app: Arc<AppContext>,
}

impl HomeAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl GetAction for HomeAction {
    fn get_route(&self) -> &str {
        "/Logs"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        None
    }

    async fn handle_request(&self, _ctx: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let mut sw = StopWatch::new();
        sw.start();
        let logs = self.app.logs.get().await;

        return super::logs::compile_result("logs", logs, sw).into();
    }
}
