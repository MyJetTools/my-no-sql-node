use std::sync::{atomic::AtomicBool, Arc};

use tokio::sync::RwLock;

use crate::tcp_client::DataReaderTcpConnection;

pub struct ConnectionToMainNode {
    connection: RwLock<Option<Arc<DataReaderTcpConnection>>>,
    has_connection: AtomicBool,
}

impl ConnectionToMainNode {
    pub fn new() -> Self {
        Self {
            connection: RwLock::new(None),
            has_connection: AtomicBool::new(false),
        }
    }

    pub async fn connected(&self, connection: Arc<DataReaderTcpConnection>) {
        let mut write_access = self.connection.write().await;
        *write_access = Some(connection);
        self.has_connection
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }

    pub async fn disconnected(&self) {
        let mut write_access = self.connection.write().await;
        *write_access = None;
        self.has_connection
            .store(false, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn has_connection(&self) -> bool {
        self.has_connection
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    pub async fn get(&self) -> Option<Arc<DataReaderTcpConnection>> {
        let read_access = self.connection.read().await;
        read_access.clone()
    }
}
