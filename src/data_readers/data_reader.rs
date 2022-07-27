use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use my_no_sql_core::db::DbTable;
use my_no_sql_tcp_shared::TcpContract;
use rust_extensions::date_time::DateTimeAsMicroseconds;
use tokio::sync::{Mutex, RwLock};

use super::{DataReaderConnection, DataReaderUpdatableData};

pub struct DataReadeMetrics {
    pub session_id: String,
    pub connected: DateTimeAsMicroseconds,
    pub last_incoming_moment: DateTimeAsMicroseconds,
    pub ip: String,
    pub name: Option<String>,
    pub tables: Vec<String>,
    pub pending_to_send: usize,
}

pub struct DataReader {
    pub id: String,
    data: RwLock<DataReaderUpdatableData>,
    pub connection: DataReaderConnection,
    has_first_init: AtomicBool,
    awating_tables: Mutex<HashMap<String, DateTimeAsMicroseconds>>,
}

impl DataReader {
    pub fn new(id: String, connection: DataReaderConnection) -> Self {
        Self {
            id,
            data: RwLock::new(DataReaderUpdatableData::new()),
            connection,
            has_first_init: AtomicBool::new(false),
            awating_tables: Mutex::new(HashMap::new()),
        }
    }

    pub async fn add_awaiting_tables(&self, table_name: String) {
        let mut write_access = self.awating_tables.lock().await;
        write_access.insert(table_name, DateTimeAsMicroseconds::now());
        //TODO - Make awaiting tables GC
    }

    pub async fn has_awaiting_table(&self, table_name: &str) -> bool {
        let mut write_access = self.awating_tables.lock().await;
        let result = write_access.remove(table_name);
        result.is_some()
    }

    pub fn has_first_init(&self) -> bool {
        self.has_first_init.load(Ordering::Relaxed)
    }

    pub fn set_first_init(&self) {
        self.has_first_init.store(true, Ordering::SeqCst);
    }

    pub async fn has_table(&self, table_name: &str) -> bool {
        let read_access = self.data.read().await;
        read_access.has_table(table_name)
    }

    pub async fn set_name_as_reader(&self, name: String) {
        self.connection.set_name_as_reader(name).await;
    }

    pub async fn set_name_as_node(&self, location: String, version: String) {
        self.connection.set_name_as_node(location, version).await;
    }

    pub async fn get_name(&self) -> Option<String> {
        self.connection.get_name().await
    }

    pub async fn subscribe(&self, db_table: Arc<DbTable>) {
        let mut write_access = self.data.write().await;
        write_access.subscribe(db_table);
    }

    pub async fn send_error_to_client(&self, message: String) {
        if let DataReaderConnection::Tcp(tcp) = &self.connection {
            let error = TcpContract::Error { message };
            tcp.send(error.serialize().as_slice()).await;
        }
    }

    fn get_ip(&self) -> String {
        match &self.connection {
            DataReaderConnection::Tcp(connection) => connection.get_ip(),
            DataReaderConnection::Http(connection) => connection.ip.to_string(),
        }
    }

    fn get_connected_moment(&self) -> DateTimeAsMicroseconds {
        match &self.connection {
            DataReaderConnection::Tcp(connection) => connection.connection.statistics.connected,
            DataReaderConnection::Http(connection) => connection.connected,
        }
    }

    pub fn get_last_incoming_moment(&self) -> DateTimeAsMicroseconds {
        match &self.connection {
            DataReaderConnection::Tcp(connection) => connection
                .connection
                .statistics
                .last_receive_moment
                .as_date_time(),
            DataReaderConnection::Http(connection) => {
                connection.last_incoming_moment.as_date_time()
            }
        }
    }

    pub async fn get_metrics(&self) -> DataReadeMetrics {
        let session_id = self.id.to_string();
        let ip = self.get_ip();
        let connected = self.get_connected_moment();
        let last_incoming_moment = self.get_last_incoming_moment();

        let pending_to_send = self.get_pending_to_send();

        let name = self.connection.get_name().await;

        let read_access = self.data.read().await;

        DataReadeMetrics {
            session_id,
            connected,
            last_incoming_moment,
            ip,
            name,
            tables: read_access.get_table_names(),
            pending_to_send,
        }
    }

    pub fn is_node(&self) -> bool {
        match &self.connection {
            DataReaderConnection::Tcp(tcp_connection) => tcp_connection.is_node(),
            DataReaderConnection::Http(_) => false,
        }
    }

    pub fn get_pending_to_send(&self) -> usize {
        match &self.connection {
            DataReaderConnection::Tcp(connection) => connection.get_pending_to_send(),
            DataReaderConnection::Http(connection) => connection.get_pending_to_send(),
        }
    }

    pub async fn ping_http_servers(&self, now: DateTimeAsMicroseconds) {
        if let DataReaderConnection::Http(info) = &self.connection {
            info.ping(now).await;
        }
    }

    pub async fn get_sent_per_second(&self) -> Vec<usize> {
        match &self.connection {
            DataReaderConnection::Tcp(tcp) => tcp.sent_per_second.get_snapshot().await,
            DataReaderConnection::Http(_) => vec![],
        }
    }
}
