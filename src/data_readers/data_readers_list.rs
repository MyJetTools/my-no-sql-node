use std::{sync::Arc, time::Duration};

use my_no_sql_sdk::core::{
    db::DbNamespaceName, rust_extensions::date_time::DateTimeAsMicroseconds,
};
use parking_lot::RwLock;

use crate::tcp_server::MyNoSqlTcpConnection;

use super::{
    data_readers_data::DataReadersData, DataReader, DataReaderConnection, HttpConnectionInfo,
    TcpConnectionInfo,
};

pub struct DataReadersList {
    data: RwLock<DataReadersData>,
    http_session_time_out: Duration,
}

impl DataReadersList {
    pub fn new(http_session_time_out: Duration) -> Self {
        Self {
            data: RwLock::new(DataReadersData::new()),
            http_session_time_out,
        }
    }

    pub fn add_tcp(&self, tcp_connection: Arc<MyNoSqlTcpConnection>) {
        let connection_id = tcp_connection.id;

        let data_reader = Arc::new(DataReader::new(
            format!("Tcp-{}", connection_id),
            DataReaderConnection::Tcp(TcpConnectionInfo::new(tcp_connection)),
        ));

        self.data.write().insert_tcp(connection_id, data_reader);
    }

    pub fn add_http(&self, ip: String) -> Arc<DataReader> {
        let mut write_access = self.data.write();

        let id = format!("Http-{}", write_access.get_next_http_id());

        let data_reader = Arc::new(DataReader::new(
            id,
            DataReaderConnection::Http(HttpConnectionInfo::new(ip)),
        ));

        write_access.insert_http(data_reader.clone());

        data_reader
    }

    pub fn get_tcp(&self, tcp_connection: &MyNoSqlTcpConnection) -> Option<Arc<DataReader>> {
        self.data.read().get_tcp(tcp_connection.id)
    }

    pub fn get_http(&self, session_id: &str) -> Option<Arc<DataReader>> {
        self.data.read().get_http(session_id)
    }

    pub fn remove_tcp(&self, tcp_connection: &MyNoSqlTcpConnection) -> Option<Arc<DataReader>> {
        self.data.write().remove_tcp(tcp_connection.id)
    }

    pub fn get_all(&self) -> Vec<Arc<DataReader>> {
        self.data.read().get_all()
    }

    pub fn get_subscribed_to_table(
        &self,
        namespace: &DbNamespaceName,
        table_name: &str,
    ) -> Vec<Arc<DataReader>> {
        self.data
            .read()
            .get_subscribed_to_table(namespace, table_name)
    }

    pub fn take_readers_awaiting_table(
        &self,
        namespace: &DbNamespaceName,
        table_name: &str,
    ) -> Vec<Arc<DataReader>> {
        self.data
            .read()
            .take_readers_awaiting_table(namespace, table_name)
    }

    pub fn gc_http_sessions(&self, now: DateTimeAsMicroseconds) -> Vec<Arc<DataReader>> {
        self.data
            .write()
            .gc_http_sessions(now, self.http_session_time_out)
    }
}
