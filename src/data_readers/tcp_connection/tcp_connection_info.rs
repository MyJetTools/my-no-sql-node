use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};

use tokio::sync::Mutex;

use crate::tcp::MyNoSqlTcpConnection;

use super::SendPerSecond;

pub enum ReaderName {
    AsReader(String),
    AsNode { location: String, version: String },
    None,
}

pub struct TcpConnectionInfo {
    pub connection: Arc<MyNoSqlTcpConnection>,
    pub name: Mutex<ReaderName>,
    sent_per_second_accumulator: AtomicUsize,
    pub sent_per_second: SendPerSecond,
    pub is_node: AtomicBool,
}

impl TcpConnectionInfo {
    pub fn new(connection: Arc<MyNoSqlTcpConnection>) -> Self {
        Self {
            connection,
            name: Mutex::new(ReaderName::None),
            sent_per_second_accumulator: AtomicUsize::new(0),
            sent_per_second: SendPerSecond::new(),
            is_node: AtomicBool::new(false),
        }
    }

    pub fn is_node(&self) -> bool {
        self.is_node.load(Ordering::Relaxed)
    }

    pub fn get_id(&self) -> i32 {
        self.connection.id
    }

    pub fn get_ip(&self) -> String {
        match &self.connection.addr {
            Some(addr) => format!("{}", addr),
            None => "unknown".to_string(),
        }
    }

    pub async fn get_name(&self) -> Option<String> {
        let read_access = self.name.lock().await;

        match &*read_access {
            ReaderName::AsReader(name) => Some(name.clone()),
            ReaderName::AsNode { location, version } => Some(format!("{}:{}", location, version)),
            ReaderName::None => None,
        }
    }

    pub async fn set_name_as_reader(&self, name: String) {
        let mut write_access = self.name.lock().await;
        *write_access = ReaderName::AsReader(name);
    }

    pub async fn set_name_as_node(&self, location: String, version: String) {
        self.is_node.store(true, Ordering::SeqCst);
        let mut write_access = self.name.lock().await;
        *write_access = ReaderName::AsNode { location, version };
    }

    pub async fn send(&self, payload_to_send: &[u8]) {
        self.sent_per_second_accumulator
            .fetch_add(payload_to_send.len(), std::sync::atomic::Ordering::SeqCst);
        self.connection.send_bytes(payload_to_send).await;
    }

    pub async fn timer_1sec_tick(&self) {
        let value = self
            .sent_per_second_accumulator
            .swap(0, std::sync::atomic::Ordering::SeqCst);
        self.sent_per_second.add(value).await;
    }

    pub fn get_pending_to_send(&self) -> usize {
        self.connection
            .statistics
            .pending_to_send_buffer_size
            .load(std::sync::atomic::Ordering::Relaxed)
    }
}
