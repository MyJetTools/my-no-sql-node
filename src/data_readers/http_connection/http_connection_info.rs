use std::sync::atomic::{AtomicUsize, Ordering};

use my_no_sql_sdk::core::rust_extensions::date_time::{
    AtomicDateTimeAsMicroseconds, DateTimeAsMicroseconds,
};
use parking_lot::Mutex;

use crate::db_sync::SyncEvent;

use super::{HttpConnectionDeliveryInfo, HttpPayload, NewRequestResult};

pub struct HttpConnectionInfo {
    pub ip: String,
    pub connected: DateTimeAsMicroseconds,
    pub last_incoming_moment: AtomicDateTimeAsMicroseconds,
    delivery_info: Mutex<HttpConnectionDeliveryInfo>,
    pending_to_send: AtomicUsize,
    name: Mutex<Option<String>>,
}

impl HttpConnectionInfo {
    pub fn new(ip: String) -> Self {
        Self {
            ip,
            connected: DateTimeAsMicroseconds::now(),
            last_incoming_moment: AtomicDateTimeAsMicroseconds::now(),
            delivery_info: Mutex::new(HttpConnectionDeliveryInfo::new()),
            pending_to_send: AtomicUsize::new(0),
            name: Mutex::new(None),
        }
    }

    pub fn get_name(&self) -> Option<String> {
        self.name.lock().clone()
    }

    pub fn set_name(&self, name: String) {
        *self.name.lock() = Some(name);
    }

    pub fn ping(&self, now: DateTimeAsMicroseconds) {
        self.delivery_info.lock().ping(now);
    }

    pub fn send(&self, sync_event: &SyncEvent) {
        // Compiled before the lock is taken - it is the heavy part.
        let payload = super::compile_http_payload(sync_event);

        let mut delivery_info = self.delivery_info.lock();
        delivery_info.upload(payload.as_slice());

        self.pending_to_send
            .store(delivery_info.get_size(), Ordering::Relaxed);
    }

    /// Waits for the changes of the session. `None` means the session is gone - it was collected
    /// after a period of silence.
    pub async fn new_request(&self) -> Option<HttpPayload> {
        let result = {
            let mut delivery_info = self.delivery_info.lock();
            let result = delivery_info.new_request(DateTimeAsMicroseconds::now());

            self.pending_to_send
                .store(delivery_info.get_size(), Ordering::Relaxed);

            result
        };

        match result {
            NewRequestResult::Payload(payload) => Some(HttpPayload::Payload(payload)),
            NewRequestResult::Await(receiver) => receiver.await.ok(),
        }
    }

    pub fn get_pending_to_send(&self) -> usize {
        self.pending_to_send.load(Ordering::Relaxed)
    }
}
