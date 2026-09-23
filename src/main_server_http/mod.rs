//! The node as a writer of the main node: what a writer sends to the node is sent on to the main
//! node's HTTP api, and the main node's answer goes back as it is. Writers connect to the node
//! exactly as they would to the main node.

mod forwarded_headers;
mod forwarder;
pub use forwarder::*;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};

use crate::app::AppContext;

/// Forwards the request to the main node. A node which is not told where the main node's HTTP
/// api is refuses - with a 400, which every writer reports as a failed write.
pub async fn forward(
    app: &AppContext,
    ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let mut guard = ForwardGuard::new(app, ctx);

    let (result, outcome) = match app.main_server_http.as_ref() {
        Some(main_server_http) => main_server_http.forward(ctx).await,
        None => (
            Err(HttpFailResult::from((
                400u16,
                "Writes are not configured on this node: MainServerHttp is not set".to_string(),
            ))),
            ForwardOutcome::NotConfigured,
        ),
    };

    guard.outcome = Some(outcome);
    result
}

/// Counts every forward - and notices the one its client gave up on: the writer's own timeout
/// drops the request, the forward with it, and no answer is ever logged.
struct ForwardGuard<'s> {
    app: &'s AppContext,
    route: String,
    outcome: Option<ForwardOutcome>,
}

impl<'s> ForwardGuard<'s> {
    fn new(app: &'s AppContext, ctx: &HttpContext) -> Self {
        Self {
            app,
            route: format!("{} {}", ctx.request.method, ctx.request.http_path.as_str()),
            outcome: None,
        }
    }
}

impl Drop for ForwardGuard<'_> {
    fn drop(&mut self) {
        let label = match self.outcome {
            Some(outcome) => outcome.as_metric_label(),
            None => {
                my_logger::LOGGER.write_warning(
                    "MainServerHttp",
                    "The client gave up before the main node answered - the request may or may not have been applied",
                    my_logger::LogEventCtx::new().add("route", self.route.clone()),
                );
                "cancelled"
            }
        };

        self.app.metrics.inc_main_server_http_request(label);
    }
}
