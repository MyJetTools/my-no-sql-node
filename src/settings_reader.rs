use my_no_sql_sdk::core::rust_extensions;
use serde::{Deserialize, Serialize};

const SETTINGS_FILE_NAME: &str = "~/.mynosqlserver-node";

const DEFAULT_HTTP_PORT: u16 = 5123;
const DEFAULT_TCP_PORT: u16 = 5125;
const DEFAULT_MAX_NAMESPACES: usize = 16;

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

    // The default namespace is one of them, and it always exists.
    if settings.get_max_namespaces() == 0 {
        return Err("MaxNamespaces can not be 0: the default namespace is one of them".to_string());
    }

    Ok(settings)
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
}
