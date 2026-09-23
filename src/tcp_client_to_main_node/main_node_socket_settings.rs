use my_tcp_sockets::{TcpClientSocketSettings, TlsSettings};

pub struct MainNodeSocketSettings {
    host_port: String,
}

impl MainNodeSocketSettings {
    pub fn new(host_port: String) -> Self {
        Self { host_port }
    }
}

#[async_trait::async_trait]
impl TcpClientSocketSettings for MainNodeSocketSettings {
    async fn get_host_port(&self) -> Option<String> {
        Some(self.host_port.clone())
    }

    async fn get_tls_settings(&self) -> Option<TlsSettings> {
        None
    }
}
