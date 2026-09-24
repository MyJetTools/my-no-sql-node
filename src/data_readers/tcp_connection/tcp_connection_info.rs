use std::sync::{
    atomic::{AtomicI64, AtomicUsize, Ordering},
    Arc,
};

use my_no_sql_sdk::tcp_contracts::MyNoSqlTcpContract;
use my_tcp_sockets::SocketAddress;
use parking_lot::Mutex;

use crate::tcp_server::MyNoSqlTcpConnection;

use super::SendPerSecond;

/// Kept in `latency_micros` until the reader reports its first round trip.
const LATENCY_NOT_MEASURED: i64 = -1;

pub struct TcpConnectionInfo {
    pub connection: Arc<MyNoSqlTcpConnection>,
    name: Mutex<Option<String>>,
    sent_per_second_accumulator: AtomicUsize,
    pub sent_per_second: SendPerSecond,
    /// Round trip of the connection in microseconds. Only the reader can measure it - it is the
    /// side which pings - so this is the number its last `PingWithLatency` carried.
    latency_micros: AtomicI64,
}

impl TcpConnectionInfo {
    pub fn new(connection: Arc<MyNoSqlTcpConnection>) -> Self {
        Self {
            connection,
            name: Mutex::new(None),
            sent_per_second_accumulator: AtomicUsize::new(0),
            sent_per_second: SendPerSecond::new(),
            latency_micros: AtomicI64::new(LATENCY_NOT_MEASURED),
        }
    }

    pub fn set_latency(&self, micros: u64) {
        let micros = i64::try_from(micros).unwrap_or(i64::MAX);
        self.latency_micros.store(micros, Ordering::Relaxed);
    }

    /// `None` until the reader reports a round trip - and for good, when its SDK predates
    /// `PingWithLatency` and it pings with a plain `Ping`.
    pub fn get_latency(&self) -> Option<i64> {
        let micros = self.latency_micros.load(Ordering::Relaxed);

        if micros < 0 {
            return None;
        }

        Some(micros)
    }

    pub fn get_ip(&self) -> String {
        // The address alone - its Display form prefixes it with the transport.
        match &self.connection.addr {
            Some(SocketAddress::Tcp(addr)) => addr.to_string(),
            Some(addr @ SocketAddress::UnixSocket(_)) => addr.to_string(),
            None => "unknown".to_string(),
        }
    }

    pub fn get_name(&self) -> Option<String> {
        self.name.lock().clone()
    }

    pub fn set_name(&self, name: String) {
        *self.name.lock() = Some(name);
    }

    pub fn send(&self, contracts: &[MyNoSqlTcpContract]) {
        let sent_amount = self.connection.send_many(contracts);

        self.sent_per_second_accumulator
            .fetch_add(sent_amount, Ordering::Relaxed);
    }

    pub fn timer_1sec_tick(&self) {
        let value = self.sent_per_second_accumulator.swap(0, Ordering::Relaxed);
        self.sent_per_second.add(value);
    }

    pub fn get_pending_to_send(&self) -> usize {
        self.connection
            .statistics()
            .pending_to_send_buffer_size
            .load(Ordering::Relaxed)
    }
}
