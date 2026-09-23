use std::{
    sync::{
        atomic::{AtomicI64, Ordering},
        Arc,
    },
    time::Duration,
};

use my_no_sql_sdk::{
    core::{db::DbNamespaceName, rust_extensions::AppStates},
    tcp_contracts::{
        sync_to_main::SyncToMainNodeHandler, MyNoSqlTcpContract, MyNoSqlTcpSerializerFactory,
    },
};
use my_tcp_sockets::TcpClient;
use parking_lot::Mutex;

use crate::{
    app::{PerSecondCounter, TrafficPerSecond},
    tcp_client_to_main_node::{MainNodeSocketSettings, TcpClientSocketCallback},
    tcp_server::MyNoSqlTcpConnection,
};

use super::main_node_connection_inner::{MainNodeConnectionInner, TableRequestState};

/// Link of one namespace to the main node.
///
/// Every namespace has a TCP connection of its own: `SetNamespace` pins a connection to its
/// namespace before the first subscription, and it can not be moved to another one afterwards.
pub struct MainNodeConnection {
    inner: Mutex<MainNodeConnectionInner<MyNoSqlTcpConnection>>,
    ping_micros: AtomicI64,
    /// Bytes of table data the main node sent. Counted here: the socket library counts only a
    /// few header bytes of every packet it receives.
    received_payload: PerSecondCounter,
    tcp_client: TcpClient,
    /// Delivers what the readers of the namespace report - expiration and last read time
    /// updates - to the main node.
    pub sync_to_main_node: SyncToMainNodeHandler,
}

impl MainNodeConnection {
    pub fn new(
        namespace: &DbNamespaceName,
        main_server: &str,
        app_states: &Arc<AppStates>,
    ) -> Self {
        let tcp_client = TcpClient::new(
            format!("MainNode[{}]", namespace),
            Arc::new(MainNodeSocketSettings::new(main_server.to_string())),
        )
        .set_app_states(app_states.clone());

        Self {
            inner: Mutex::new(MainNodeConnectionInner::new()),
            ping_micros: AtomicI64::new(0),
            received_payload: PerSecondCounter::default(),
            tcp_client,
            sync_to_main_node: SyncToMainNodeHandler::new(),
        }
    }

    pub async fn start(
        &self,
        socket_callback: TcpClientSocketCallback,
        app_states: Arc<AppStates>,
    ) {
        self.sync_to_main_node
            .start(my_logger::LOGGER.clone(), app_states);

        self.tcp_client
            .start(
                Arc::new(MyNoSqlTcpSerializerFactory),
                socket_callback,
                my_logger::LOGGER.clone(),
            )
            .await;
    }

    pub fn is_connected(&self) -> bool {
        self.inner.lock().is_connected()
    }

    pub fn add_received_payload(&self, size: usize) {
        self.received_payload.add(size);
    }

    pub fn one_second_tick(&self) {
        self.received_payload.one_second_tick();
    }

    /// Incoming - the table data the main node sent; outgoing - everything the node sent over
    /// the current connection. Nothing while it is disconnected.
    pub fn get_traffic_per_second(&self) -> TrafficPerSecond {
        let outgoing = match self.inner.lock().get_connection() {
            Some(connection) => connection.statistics().sent_per_sec.get_value(),
            None => 0,
        };

        TrafficPerSecond {
            incoming: self.received_payload.get(),
            outgoing,
        }
    }

    /// Every table the readers of the namespace subscribed to, with what the main node answered.
    pub fn get_requested_tables(&self) -> Vec<(String, TableRequestState)> {
        self.inner.lock().get_requested_tables()
    }

    pub fn get_ping_micros(&self) -> i64 {
        self.ping_micros.load(Ordering::Relaxed)
    }

    pub fn update_ping(&self, ping_duration: Duration) {
        self.ping_micros
            .store(ping_duration.as_micros() as i64, Ordering::Relaxed);
    }

    /// Asks the main node for the table, whatever the state of the connection is: a table asked
    /// for while disconnected is asked for as soon as the connection is established.
    pub fn request_table(&self, table_name: &str) {
        // Sent under the lock: a request racing with a reconnect is neither lost nor sent over a
        // connection the main node has not been greeted on yet.
        let mut inner = self.inner.lock();

        if let Some(connection) = inner.request_table(table_name) {
            send_subscribe_as_node(connection.as_ref(), table_name);
        }
    }

    /// Called once the connection is greeted and pinned to the namespace.
    pub fn connected(&self, connection: &Arc<MyNoSqlTcpConnection>) {
        let mut inner = self.inner.lock();

        for table_name in inner.connected(connection.clone()) {
            send_subscribe_as_node(connection.as_ref(), table_name.as_str());
        }
    }

    pub fn disconnected(&self) {
        self.inner.lock().disconnected();
        // A link which is down has no latency - the last one measured would read as a live one.
        self.ping_micros.store(0, Ordering::Relaxed);
    }

    pub fn table_replicated(&self, table_name: &str) {
        self.inner.lock().table_replicated(table_name);
    }

    pub fn table_not_found(&self, table_name: &str) {
        self.inner.lock().table_not_found(table_name);
    }

    pub fn retry_not_found_tables(&self) {
        let inner = self.inner.lock();

        if let Some(to_retry) = inner.get_tables_to_retry() {
            for table_name in to_retry.tables.iter() {
                send_subscribe_as_node(to_retry.connection.as_ref(), table_name.as_str());
            }
        }
    }
}

fn send_subscribe_as_node(connection: &MyNoSqlTcpConnection, table_name: &str) {
    connection.send(&MyNoSqlTcpContract::SubscribeAsNode(table_name.to_string()));
}
