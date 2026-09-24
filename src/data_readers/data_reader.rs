use std::sync::atomic::{AtomicBool, Ordering};

use my_no_sql_sdk::core::{
    db::DbNamespaceName, rust_extensions::date_time::DateTimeAsMicroseconds,
};
use parking_lot::RwLock;

use super::{data_reader_updatable_data::DataReaderUpdatableData, DataReaderConnection};

pub struct DataReaderMetrics {
    pub session_id: String,
    pub namespace: String,
    pub connected: DateTimeAsMicroseconds,
    pub last_incoming_moment: DateTimeAsMicroseconds,
    pub ip: String,
    pub name: Option<String>,
    pub tables: Vec<String>,
    /// Tables the reader subscribed to which have not arrived from the main node yet.
    pub awaiting_tables: Vec<String>,
    pub pending_to_send: usize,
    /// Round trip in microseconds, as the reader reported it. `None` until it does.
    pub latency: Option<i64>,
}

pub struct DataReader {
    pub id: String,
    data: RwLock<DataReaderUpdatableData>,
    pub connection: DataReaderConnection,
    has_first_init: AtomicBool,
}

impl DataReader {
    pub fn new(id: String, connection: DataReaderConnection) -> Self {
        Self {
            id,
            data: RwLock::new(DataReaderUpdatableData::new()),
            connection,
            has_first_init: AtomicBool::new(false),
        }
    }

    pub fn get_namespace(&self) -> DbNamespaceName {
        self.data.read().get_namespace().clone()
    }

    pub fn set_namespace(&self, namespace: DbNamespaceName) -> Result<(), String> {
        self.data.write().set_namespace(namespace)
    }

    pub fn subscribe(&self, namespace: &DbNamespaceName, table_name: &str) -> bool {
        self.data.write().subscribe(namespace, table_name)
    }

    pub fn unsubscribe(&self, table_name: &str) {
        self.data.write().unsubscribe(table_name);
    }

    pub fn is_subscribed_to(&self, namespace: &DbNamespaceName, table_name: &str) -> bool {
        self.data.read().is_subscribed_to(namespace, table_name)
    }

    pub fn add_awaiting_table(&self, namespace: &DbNamespaceName, table_name: &str) -> bool {
        self.data.write().add_awaiting_table(namespace, table_name)
    }

    pub fn take_awaiting_table(&self, namespace: &DbNamespaceName, table_name: &str) -> bool {
        self.data.write().take_awaiting_table(namespace, table_name)
    }

    pub fn has_first_init(&self) -> bool {
        self.has_first_init.load(Ordering::Relaxed)
    }

    pub fn set_first_init(&self) {
        self.has_first_init.store(true, Ordering::SeqCst);
    }

    pub fn get_name(&self) -> Option<String> {
        self.connection.get_name()
    }

    pub fn set_name(&self, name: String) {
        self.connection.set_name(name);
    }

    fn get_ip(&self) -> String {
        match &self.connection {
            DataReaderConnection::Tcp(connection) => connection.get_ip(),
            DataReaderConnection::Http(connection) => connection.ip.to_string(),
        }
    }

    fn get_connected_moment(&self) -> DateTimeAsMicroseconds {
        match &self.connection {
            DataReaderConnection::Tcp(connection) => connection.connection.statistics().connected,
            DataReaderConnection::Http(connection) => connection.connected,
        }
    }

    pub fn get_last_incoming_moment(&self) -> DateTimeAsMicroseconds {
        match &self.connection {
            DataReaderConnection::Tcp(connection) => connection
                .connection
                .statistics()
                .last_receive_moment
                .as_date_time(),
            DataReaderConnection::Http(connection) => {
                connection.last_incoming_moment.as_date_time()
            }
        }
    }

    pub fn get_metrics(&self) -> DataReaderMetrics {
        let (namespace, tables, awaiting_tables) = {
            let read_access = self.data.read();
            (
                read_access.get_namespace().to_string(),
                read_access.get_table_names(),
                read_access.get_awaiting_table_names(),
            )
        };

        DataReaderMetrics {
            session_id: self.id.to_string(),
            namespace,
            connected: self.get_connected_moment(),
            last_incoming_moment: self.get_last_incoming_moment(),
            ip: self.get_ip(),
            name: self.get_name(),
            tables,
            awaiting_tables,
            pending_to_send: self.get_pending_to_send(),
            latency: self.get_latency(),
        }
    }

    /// Kept for a TCP reader only - an HTTP reader does not ping.
    pub fn set_latency(&self, micros: u64) {
        match &self.connection {
            DataReaderConnection::Tcp(connection) => connection.set_latency(micros),
            DataReaderConnection::Http(_) => {}
        }
    }

    /// `None` for an HTTP reader, and for a TCP one until it reports its first round trip.
    pub fn get_latency(&self) -> Option<i64> {
        match &self.connection {
            DataReaderConnection::Tcp(connection) => connection.get_latency(),
            DataReaderConnection::Http(_) => None,
        }
    }

    pub fn get_pending_to_send(&self) -> usize {
        match &self.connection {
            DataReaderConnection::Tcp(connection) => connection.get_pending_to_send(),
            DataReaderConnection::Http(connection) => connection.get_pending_to_send(),
        }
    }

    pub fn ping_http_session(&self, now: DateTimeAsMicroseconds) {
        if let DataReaderConnection::Http(info) = &self.connection {
            info.ping(now);
        }
    }

    /// Bytes sent to the reader during the last second. `None` for an HTTP reader: its
    /// requests are served by the http server, which does not count bytes per session - there
    /// is no number to show, not a zero.
    ///
    /// What a reader sends is not reported: the socket library counts only a few header bytes
    /// of every packet it receives.
    pub fn get_outgoing_per_second(&self) -> Option<usize> {
        match &self.connection {
            DataReaderConnection::Tcp(tcp) => {
                Some(tcp.connection.statistics().sent_per_sec.get_value())
            }
            DataReaderConnection::Http(_) => None,
        }
    }

    pub fn get_sent_per_second(&self) -> Vec<usize> {
        match &self.connection {
            DataReaderConnection::Tcp(tcp) => tcp.sent_per_second.get_snapshot(),
            DataReaderConnection::Http(_) => vec![],
        }
    }
}
