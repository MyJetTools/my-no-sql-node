use super::{HttpConnectionInfo, TcpConnectionInfo};

pub enum DataReaderConnection {
    Tcp(TcpConnectionInfo),
    Http(HttpConnectionInfo),
}

impl DataReaderConnection {
    pub fn get_name(&self) -> Option<String> {
        match self {
            DataReaderConnection::Tcp(tcp_info) => tcp_info.get_name(),
            DataReaderConnection::Http(http_info) => http_info.get_name(),
        }
    }

    pub fn set_name(&self, name: String) {
        match self {
            DataReaderConnection::Tcp(tcp_info) => tcp_info.set_name(name),
            DataReaderConnection::Http(http_info) => http_info.set_name(name),
        }
    }

    pub fn one_sec_tick(&self) {
        match self {
            DataReaderConnection::Tcp(tcp_info) => tcp_info.timer_1sec_tick(),
            DataReaderConnection::Http(_) => {}
        }
    }
}

impl crate::app::UpdatePendingToSyncModel for DataReaderConnection {
    fn get_name(&self) -> Option<String> {
        DataReaderConnection::get_name(self)
    }

    fn get_pending_to_sync(&self) -> usize {
        match self {
            DataReaderConnection::Tcp(tcp_info) => tcp_info.get_pending_to_send(),
            DataReaderConnection::Http(http_info) => http_info.get_pending_to_send(),
        }
    }
}
