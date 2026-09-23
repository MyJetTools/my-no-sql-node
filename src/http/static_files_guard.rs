use my_http_server::{
    async_trait, HttpContext, HttpFailResult, HttpOkResult, HttpServerMiddleware,
};

/// Stands in front of the static files and lets through only the files the web UI is built of:
/// `/favicon.ico`, `/favicon.svg` and `/assets/<file>`. Anything else is a 404 right here.
///
/// The static files middleware maps the request path onto the disk as it is, `..` included, so
/// without this guard `GET /../<any file>` would read any file the process can read - the settings
/// file with its connection strings among them.
pub struct StaticFilesGuard;

#[async_trait::async_trait]
impl HttpServerMiddleware for StaticFilesGuard {
    async fn handle_request(
        &self,
        ctx: &mut HttpContext,
    ) -> Option<Result<HttpOkResult, HttpFailResult>> {
        let method = &ctx.request.method;
        let is_read = *method == my_http_server::hyper::Method::GET
            || *method == my_http_server::hyper::Method::HEAD;

        if is_read && is_ui_file(ctx.request.http_path.as_str()) {
            return None;
        }

        Some(Err(HttpFailResult::as_not_found(
            "Not found".to_string(),
            false,
        )))
    }
}

fn is_ui_file(path: &str) -> bool {
    if path == "/favicon.ico" || path == "/favicon.svg" {
        return true;
    }

    match path.strip_prefix("/assets/") {
        Some(file_name) => is_plain_file_name(file_name),
        None => false,
    }
}

/// One path segment of a file name: no directories, no `..`, no escapes.
fn is_plain_file_name(src: &str) -> bool {
    !src.is_empty()
        && !src.starts_with('.')
        && src
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-' || b == b'_')
}

#[cfg(test)]
mod tests {
    use super::is_ui_file;

    #[test]
    fn test_ui_files_pass() {
        assert!(is_ui_file("/favicon.ico"));
        assert!(is_ui_file("/favicon.svg"));
        assert!(is_ui_file("/assets/app.css"));
        assert!(is_ui_file("/assets/my-no-sql-node-ui_bg-dxh74b54c1d080e7c7.wasm"));
    }

    #[test]
    fn test_anything_else_is_refused() {
        assert!(!is_ui_file("/../Cargo.toml"));
        assert!(!is_ui_file("/assets/../../Cargo.toml"));
        assert!(!is_ui_file("/assets/..%2F..%2FCargo.toml"));
        assert!(!is_ui_file("/assets/.."));
        assert!(!is_ui_file("/assets/.hidden"));
        assert!(!is_ui_file("/assets/"));
        assert!(!is_ui_file("/assets/img/logo.png"));
        assert!(!is_ui_file("/assets\\..\\Cargo.toml"));
        assert!(!is_ui_file("/index.html"));
        assert!(!is_ui_file("/Cargo.toml"));
        assert!(!is_ui_file("/etc/passwd"));
    }
}
