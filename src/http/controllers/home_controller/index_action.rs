use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput, WebContentType};
use my_http_server_controllers::controllers::{
    actions::GetAction, documentation::HttpActionDescription,
};

use crate::app::AppContext;

pub struct IndexAction {
    pub app: Arc<AppContext>,
}

impl IndexAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl GetAction for IndexAction {
    fn get_route(&self) -> &str {
        "/"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        None
    }

    async fn handle_request(&self, _: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let content = format!(
            r###"<html><head><title>{} MyNoSQLServer</title>
            <link href="/css/bootstrap.css" rel="stylesheet" type="text/css" />
            <link href="/css/site.css" rel="stylesheet" type="text/css" />
            <script src="/js/jquery.js"></script><script src="/js/app.js?ver={rnd}"></script>
            </head><body></body></html>"###,
            ver = crate::app::APP_VERSION,
            rnd = self.app.process_id
        );

        HttpOutput::Content {
            headers: None,
            content_type: Some(WebContentType::Html),
            content: content.into_bytes(),
        }
        .into_ok_result(true)
        .into()
    }
}
