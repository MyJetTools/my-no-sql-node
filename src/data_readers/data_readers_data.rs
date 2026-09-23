use std::{collections::BTreeMap, sync::Arc, time::Duration};

use my_no_sql_sdk::core::{
    db::DbNamespaceName, rust_extensions::date_time::DateTimeAsMicroseconds,
};

use super::DataReader;

pub(super) struct DataReadersData {
    tcp: BTreeMap<i32, Arc<DataReader>>,
    http: BTreeMap<String, Arc<DataReader>>,
    next_http_id: usize,
}

impl DataReadersData {
    pub fn new() -> Self {
        Self {
            tcp: BTreeMap::new(),
            http: BTreeMap::new(),
            next_http_id: 0,
        }
    }

    pub fn get_next_http_id(&mut self) -> usize {
        let result = self.next_http_id;
        self.next_http_id += 1;
        result
    }

    pub fn insert_tcp(&mut self, connection_id: i32, data_reader: Arc<DataReader>) {
        self.tcp.insert(connection_id, data_reader);
    }

    pub fn insert_http(&mut self, data_reader: Arc<DataReader>) {
        self.http.insert(data_reader.id.to_string(), data_reader);
    }

    pub fn get_tcp(&self, connection_id: i32) -> Option<Arc<DataReader>> {
        self.tcp.get(&connection_id).cloned()
    }

    pub fn get_http(&self, session_id: &str) -> Option<Arc<DataReader>> {
        self.http.get(session_id).cloned()
    }

    pub fn remove_tcp(&mut self, connection_id: i32) -> Option<Arc<DataReader>> {
        self.tcp.remove(&connection_id)
    }

    pub fn get_all(&self) -> Vec<Arc<DataReader>> {
        self.tcp
            .values()
            .chain(self.http.values())
            .cloned()
            .collect()
    }

    pub fn get_subscribed_to_table(
        &self,
        namespace: &DbNamespaceName,
        table_name: &str,
    ) -> Vec<Arc<DataReader>> {
        self.tcp
            .values()
            .chain(self.http.values())
            .filter(|data_reader| data_reader.is_subscribed_to(namespace, table_name))
            .cloned()
            .collect()
    }

    /// Readers of the namespace which wait for the table. Each one is handed out once: the
    /// awaited table is taken off the reader in the same step.
    pub fn take_readers_awaiting_table(
        &self,
        namespace: &DbNamespaceName,
        table_name: &str,
    ) -> Vec<Arc<DataReader>> {
        self.tcp
            .values()
            .chain(self.http.values())
            .filter(|data_reader| data_reader.take_awaiting_table(namespace, table_name))
            .cloned()
            .collect()
    }

    pub fn gc_http_sessions(
        &mut self,
        now: DateTimeAsMicroseconds,
        http_timeout: Duration,
    ) -> Vec<Arc<DataReader>> {
        let expired: Vec<String> = self
            .http
            .iter()
            .filter(|(_, data_reader)| {
                now.duration_since(data_reader.get_last_incoming_moment())
                    .as_positive_or_zero()
                    >= http_timeout
            })
            .map(|(session_id, _)| session_id.to_string())
            .collect();

        expired
            .iter()
            .filter_map(|session_id| self.http.remove(session_id))
            .collect()
    }
}
