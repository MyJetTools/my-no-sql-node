use my_http_server::{
    async_trait, HttpContext, HttpFailResult, HttpOkResult, HttpOutput, HttpServerMiddleware,
};

const INDEX_FILE: &str = "./wwwroot/index.html";

/// Serves the web UI's `index.html` on the routes the UI owns, so a deep link - or a page
/// reloaded anywhere in the UI - opens the UI rather than a 404.
///
/// Only these routes and only GET (and HEAD): there is no catch-all "not found -> index.html".
/// With one, a mistyped api call - of any method - would be answered with 200 and a page of html
/// instead of a 404 the caller can notice.
pub struct UiRoutesMiddleware;

#[async_trait::async_trait]
impl HttpServerMiddleware for UiRoutesMiddleware {
    async fn handle_request(
        &self,
        ctx: &mut HttpContext,
    ) -> Option<Result<HttpOkResult, HttpFailResult>> {
        let method = &ctx.request.method;
        if *method != my_http_server::hyper::Method::GET
            && *method != my_http_server::hyper::Method::HEAD
        {
            return None;
        }

        if !is_ui_route(ctx.request.http_path.as_str()) {
            return None;
        }

        // The file is tiny. No file - no UI was built into this image - is a plain 404.
        let content = tokio::fs::read_to_string(INDEX_FILE).await.ok()?;

        Some(
            HttpOutput::as_html(content)
                // The page names the hashed wasm/js bundles - a cached copy would keep loading
                // the previous build after an upgrade.
                .add_header("Cache-Control", "no-cache")
                .into_ok_result(false),
        )
    }
}

/// The routes of the UI router (`ui/src/main.rs`): `/`, `/connections`, `/data` and
/// `/data/{table}[/{partition}[/{row}]]`.
fn is_ui_route(path: &str) -> bool {
    let path = path.trim_start_matches('/').trim_end_matches('/');

    if path.is_empty() {
        return true;
    }

    let mut segments = path.split('/');

    let Some(first) = segments.next() else {
        return true;
    };

    if first.eq_ignore_ascii_case("connections") {
        return segments.next().is_none();
    }

    if first.eq_ignore_ascii_case("data") {
        let rest: Vec<&str> = segments.collect();
        return rest.len() <= 3 && rest.iter().all(|segment| !segment.is_empty());
    }

    false
}

#[cfg(test)]
mod tests {
    use super::is_ui_route;

    #[test]
    fn test_ui_routes() {
        assert!(is_ui_route("/"));
        assert!(is_ui_route("/connections"));
        assert!(is_ui_route("/Connections/"));
        assert!(is_ui_route("/data"));
        assert!(is_ui_route("/data/"));
        assert!(is_ui_route("/data/my-table"));
        assert!(is_ui_route("/data/my-table/pk"));
        assert!(is_ui_route("/data/my-table/pk/rk"));
        assert!(is_ui_route("/Data/my%20table"));
    }

    #[test]
    fn test_not_ui_routes() {
        assert!(!is_ui_route("/api/Row"));
        assert!(!is_ui_route("/api/unknown"));
        assert!(!is_ui_route("/connections/extra"));
        assert!(!is_ui_route("/data/t/p/r/extra"));
        assert!(!is_ui_route("/data//pk"));
        assert!(!is_ui_route("/assets/app.css"));
        assert!(!is_ui_route("/metrics"));
    }
}
