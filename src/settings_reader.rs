use my_no_sql_sdk::core::rust_extensions;
use serde::{Deserialize, Serialize};

const SETTINGS_FILE_NAME: &str = "~/.mynosqlserver-node";

const DEFAULT_HTTP_PORT: u16 = 5123;
const DEFAULT_TCP_PORT: u16 = 5125;
const DEFAULT_MAX_NAMESPACES: usize = 16;
/// Below the SDK writer's own 10 s: the node has to give up first. A writer which gives up first
/// cancels the forward, the node's timeout never fires, and a connection to the main node which
/// died silently is never recognized as dead.
const DEFAULT_MAIN_SERVER_HTTP_TIMEOUT_SEC: u64 = 8;

#[derive(Serialize, Deserialize, Debug)]
pub struct SettingsModel {
    /// Name the node introduces itself with to the main node.
    #[serde(rename = "Location")]
    pub location: String,

    /// Reader TCP endpoint of the main node - `host:port`.
    #[serde(rename = "MainServer")]
    pub main_server: String,

    /// Asks the main node to compress what it sends to this node.
    #[serde(rename = "Compress")]
    pub compress: bool,

    #[serde(rename = "HttpPort")]
    pub http_port: Option<u16>,

    #[serde(rename = "TcpPort")]
    pub tcp_port: Option<u16>,

    /// Every namespace the node replicates costs a connection to the main node, and lives until
    /// the node restarts - a reader naming yet another namespace past this amount is refused.
    #[serde(rename = "MaxNamespaces")]
    pub max_namespaces: Option<usize>,

    /// HTTP api of the main node - `http://host:5123` or `https://...`. The node forwards to it
    /// every write, and everything a writer asks: writers connect to the node exactly as to the
    /// main node. Absent - the node refuses writes.
    #[serde(rename = "MainServerHttp")]
    pub main_server_http: Option<String>,

    /// Gzips the bodies of the forwarded requests. The main node has to decode request bodies -
    /// turn it on only once it does.
    #[serde(rename = "CompressWrites")]
    pub compress_writes: Option<bool>,

    /// How long a forwarded request may wait for the main node's answer, seconds. Has to stay
    /// below the timeout of the writers (10 s for the SDK writer).
    #[serde(rename = "MainServerHttpTimeoutSec")]
    pub main_server_http_timeout_sec: Option<u64>,
}

impl SettingsModel {
    pub fn get_http_port(&self) -> u16 {
        self.http_port.unwrap_or(DEFAULT_HTTP_PORT)
    }

    pub fn get_tcp_port(&self) -> u16 {
        self.tcp_port.unwrap_or(DEFAULT_TCP_PORT)
    }

    pub fn get_max_namespaces(&self) -> usize {
        self.max_namespaces.unwrap_or(DEFAULT_MAX_NAMESPACES)
    }

    pub fn get_compress_writes(&self) -> bool {
        self.compress_writes.unwrap_or(false)
    }

    pub fn get_main_server_http_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(
            self.main_server_http_timeout_sec
                .unwrap_or(DEFAULT_MAIN_SERVER_HTTP_TIMEOUT_SEC),
        )
    }
}

pub async fn read_settings() -> SettingsModel {
    let file_name = rust_extensions::file_utils::format_path(SETTINGS_FILE_NAME);

    let file_content = match tokio::fs::read(file_name.as_str()).await {
        Ok(file_content) => file_content,
        Err(err) => panic!(
            "Can not read settings file [{}]. Err: {}",
            file_name.as_str(),
            err
        ),
    };

    match parse_settings(file_content.as_slice()) {
        Ok(settings) => settings,
        Err(err) => panic!("Invalid settings file [{}]. {}", file_name.as_str(), err),
    }
}

fn parse_settings(src: &[u8]) -> Result<SettingsModel, String> {
    let mut settings: SettingsModel = match serde_yaml::from_slice(src) {
        Ok(settings) => settings,
        Err(err) => return Err(format!("Can not parse yaml. Err: {}", err)),
    };

    settings.main_server = parse_main_server(settings.main_server.as_str())?;

    if let Some(main_server_http) = settings.main_server_http.take() {
        settings.main_server_http = Some(parse_main_server_http(main_server_http.as_str())?);
    }

    if settings.main_server_http_timeout_sec == Some(0) {
        return Err("MainServerHttpTimeoutSec can not be 0".to_string());
    }

    // The default namespace is one of them, and it always exists.
    if settings.get_max_namespaces() == 0 {
        return Err("MaxNamespaces can not be 0: the default namespace is one of them".to_string());
    }

    Ok(settings)
}

/// The base url requests are forwarded to: a scheme, a host, maybe a path prefix, no trailing `/`
/// - the path of each request is appended to it as it came.
fn parse_main_server_http(src: &str) -> Result<String, String> {
    let src = src.trim().trim_end_matches('/');

    let Some((_, rest)) = src
        .split_once("://")
        .filter(|(scheme, _)| *scheme == "http" || *scheme == "https")
    else {
        return Err(
            "Invalid MainServerHttp. It must be the http api url of the main node: http://host:port or https://..."
                .to_string(),
        );
    };

    let authority = rest.split('/').next().unwrap_or_default();

    // Not repeated in the message: it would be the password.
    if authority.contains('@') {
        return Err(
            "Invalid MainServerHttp: credentials in the url are not supported".to_string(),
        );
    }

    if src.contains('?') || src.contains('#') {
        return Err(format!(
            "Invalid MainServerHttp '{}': a query or a fragment can not prefix the forwarded requests",
            src
        ));
    }

    if let Err(err) = flurl::FlUrl::try_new(src) {
        return Err(format!("Invalid MainServerHttp '{}': {:?}", src, err));
    }

    Ok(src.to_string())
}

/// The node opens a connection of its own for every namespace its readers work in, so
/// `MainServer` names the server only - a connection string which fixes a namespace would pin
/// every one of those connections to it.
fn parse_main_server(src: &str) -> Result<String, String> {
    let connection_string = match my_no_sql_sdk::parse_connection_string(src) {
        Ok(connection_string) => connection_string,
        Err(err) => return Err(format!("Invalid MainServer '{}'. {}", src, err)),
    };

    if connection_string.namespace.is_some() {
        return Err(format!(
            "MainServer '{}' must not name a namespace: the node replicates every namespace its readers subscribe in",
            src
        ));
    }

    Ok(connection_string.host)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_legacy_host_port() {
        let settings =
            parse_settings(b"Location: eu\nMainServer: 10.0.0.1:5125\nCompress: true\n").unwrap();

        assert_eq!("eu", settings.location);
        assert_eq!("10.0.0.1:5125", settings.main_server);
        assert!(settings.compress);
        assert_eq!(DEFAULT_HTTP_PORT, settings.get_http_port());
        assert_eq!(DEFAULT_TCP_PORT, settings.get_tcp_port());
        assert_eq!(DEFAULT_MAX_NAMESPACES, settings.get_max_namespaces());
    }

    #[test]
    fn test_zero_max_namespaces_is_refused() {
        let result = parse_settings(
            b"Location: eu\nMainServer: 10.0.0.1:5125\nCompress: false\nMaxNamespaces: 0\n",
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_connection_string_without_namespace() {
        let settings = parse_settings(
            b"Location: eu\nMainServer: host=10.0.0.1:5125\nCompress: false\nHttpPort: 6123\nTcpPort: 6125\n",
        )
        .unwrap();

        assert_eq!("10.0.0.1:5125", settings.main_server);
        assert_eq!(6123, settings.get_http_port());
        assert_eq!(6125, settings.get_tcp_port());
    }

    #[test]
    fn test_connection_string_with_namespace_is_refused() {
        let result = parse_settings(
            b"Location: eu\nMainServer: host=10.0.0.1:5125;ns=alpha\nCompress: false\n",
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_missing_field_is_refused() {
        let result = parse_settings(b"Location: eu\nCompress: false\n");

        assert!(result.is_err());
    }

    #[test]
    fn test_main_server_http_is_normalized() {
        let settings = parse_settings(
            b"Location: node\nMainServer: 10.0.0.1:5125\nCompress: true\nMainServerHttp: http://10.0.0.1:5123/\n",
        )
        .unwrap();

        assert_eq!(Some("http://10.0.0.1:5123"), settings.main_server_http.as_deref());
        assert!(!settings.get_compress_writes());
    }

    #[test]
    fn test_main_server_http_without_scheme_is_refused() {
        let result = parse_settings(
            b"Location: node\nMainServer: 10.0.0.1:5125\nCompress: true\nMainServerHttp: 10.0.0.1:5123\n",
        );

        assert!(result.is_err());
    }

    fn parse_with_main_server_http(main_server_http: &str) -> Result<SettingsModel, String> {
        parse_settings(
            format!(
                "Location: node\nMainServer: 10.0.0.1:5125\nCompress: true\nMainServerHttp: {}\n",
                main_server_http
            )
            .as_bytes(),
        )
    }

    #[test]
    fn test_main_server_http_with_credentials_is_refused_without_echoing_them() {
        let err = parse_with_main_server_http("http://user:secret@10.0.0.1:5123")
            .err()
            .unwrap();

        assert!(!err.contains("secret"), "{err}");
    }

    #[test]
    fn test_main_server_http_with_query_is_refused() {
        assert!(parse_with_main_server_http("http://10.0.0.1:5123?x=1").is_err());
    }

    #[test]
    fn test_main_server_http_with_path_prefix_is_kept() {
        let settings = parse_with_main_server_http("https://gw.example.com/nosql/").unwrap();

        assert_eq!(
            Some("https://gw.example.com/nosql"),
            settings.main_server_http.as_deref()
        );
    }

    #[test]
    fn test_zero_main_server_http_timeout_is_refused() {
        let result = parse_settings(
            b"Location: node\nMainServer: 10.0.0.1:5125\nCompress: true\nMainServerHttpTimeoutSec: 0\n",
        );

        assert!(result.is_err());
    }
}
