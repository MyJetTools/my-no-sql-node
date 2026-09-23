use std::time::Duration;

use my_no_sql_sdk::core::rust_extensions::date_time::DateTimeAsMicroseconds;
use tokio::sync::oneshot;

pub enum HttpPayload {
    Ping,
    Payload(Vec<u8>),
}

pub enum NewRequestResult {
    Payload(Vec<u8>),
    Await(oneshot::Receiver<HttpPayload>),
}

struct AwaitingResponse {
    created: DateTimeAsMicroseconds,
    sender: oneshot::Sender<HttpPayload>,
}

/// A GetChanges request which waited this long without changes is answered with a Ping.
const MAX_AWAITING_DURATION: Duration = Duration::from_secs(3);

/// Long polling of an HTTP reader: the changes accumulated for it, and the GetChanges request
/// waiting for them, if there is one.
pub struct HttpConnectionDeliveryInfo {
    awaiting_response: Option<AwaitingResponse>,
    payload_to_deliver: Vec<u8>,
}

impl HttpConnectionDeliveryInfo {
    pub fn new() -> Self {
        Self {
            awaiting_response: None,
            payload_to_deliver: Vec::new(),
        }
    }

    pub fn get_size(&self) -> usize {
        self.payload_to_deliver.len()
    }

    /// Accumulates the changes and hands everything accumulated to the waiting request, if any.
    pub fn upload(&mut self, payload: &[u8]) {
        self.payload_to_deliver.extend_from_slice(payload);

        let Some(awaiting_response) = self.awaiting_response.take() else {
            return;
        };

        let payload = std::mem::take(&mut self.payload_to_deliver);

        // The request is gone - its client stopped waiting. The changes stay for the next one.
        if let Err(HttpPayload::Payload(payload)) =
            awaiting_response.sender.send(HttpPayload::Payload(payload))
        {
            self.payload_to_deliver = payload;
        }
    }

    /// Gives a new GetChanges request what has been accumulated, or makes it the request which
    /// waits for the changes. Only the latest request of a session waits: the one which was
    /// waiting before is answered with a Ping.
    pub fn new_request(&mut self, now: DateTimeAsMicroseconds) -> NewRequestResult {
        if !self.payload_to_deliver.is_empty() {
            return NewRequestResult::Payload(std::mem::take(&mut self.payload_to_deliver));
        }

        if let Some(previous) = self.awaiting_response.take() {
            let _ = previous.sender.send(HttpPayload::Ping);
        }

        let (sender, receiver) = oneshot::channel();

        self.awaiting_response = Some(AwaitingResponse {
            created: now,
            sender,
        });

        NewRequestResult::Await(receiver)
    }

    /// Answers the request which waited long enough with a Ping, so it is not cut off by a proxy
    /// or by the client's own timeout.
    pub fn ping(&mut self, now: DateTimeAsMicroseconds) {
        let waited_long_enough = match &self.awaiting_response {
            Some(awaiting_response) => {
                now.duration_since(awaiting_response.created)
                    .as_positive_or_zero()
                    >= MAX_AWAITING_DURATION
            }
            None => false,
        };

        if !waited_long_enough {
            return;
        }

        if let Some(awaiting_response) = self.awaiting_response.take() {
            let _ = awaiting_response.sender.send(HttpPayload::Ping);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use my_no_sql_sdk::core::rust_extensions::date_time::DateTimeAsMicroseconds;

    use super::{HttpConnectionDeliveryInfo, HttpPayload, NewRequestResult};

    fn unwrap_payload(payload: HttpPayload) -> Vec<u8> {
        match payload {
            HttpPayload::Payload(payload) => payload,
            HttpPayload::Ping => panic!("Payload is expected"),
        }
    }

    #[tokio::test]
    async fn test_accumulated_changes_are_given_to_the_next_request() {
        let mut delivery_info = HttpConnectionDeliveryInfo::new();

        delivery_info.upload(&[1, 2]);
        delivery_info.upload(&[3]);

        match delivery_info.new_request(DateTimeAsMicroseconds::now()) {
            NewRequestResult::Payload(payload) => assert_eq!(vec![1, 2, 3], payload),
            NewRequestResult::Await(_) => panic!("Payload is expected"),
        }

        assert_eq!(0, delivery_info.get_size());
    }

    #[tokio::test]
    async fn test_waiting_request_gets_the_changes() {
        let mut delivery_info = HttpConnectionDeliveryInfo::new();

        let NewRequestResult::Await(receiver) =
            delivery_info.new_request(DateTimeAsMicroseconds::now())
        else {
            panic!("The request has to wait");
        };

        delivery_info.upload(&[1, 2, 3]);

        assert_eq!(vec![1, 2, 3], unwrap_payload(receiver.await.unwrap()));
        assert_eq!(0, delivery_info.get_size());
    }

    #[tokio::test]
    async fn test_changes_are_kept_when_the_waiting_request_is_gone() {
        let mut delivery_info = HttpConnectionDeliveryInfo::new();

        let receiver = delivery_info.new_request(DateTimeAsMicroseconds::now());
        drop(receiver);

        delivery_info.upload(&[1, 2, 3]);

        assert_eq!(3, delivery_info.get_size());
    }

    #[tokio::test]
    async fn test_previous_request_is_answered_with_ping() {
        let mut delivery_info = HttpConnectionDeliveryInfo::new();

        let NewRequestResult::Await(first) =
            delivery_info.new_request(DateTimeAsMicroseconds::now())
        else {
            panic!("The request has to wait");
        };

        let _second = delivery_info.new_request(DateTimeAsMicroseconds::now());

        assert!(matches!(first.await.unwrap(), HttpPayload::Ping));
    }

    #[tokio::test]
    async fn test_request_which_waited_long_enough_is_pinged() {
        let mut delivery_info = HttpConnectionDeliveryInfo::new();

        let created = DateTimeAsMicroseconds::now();

        let NewRequestResult::Await(receiver) = delivery_info.new_request(created) else {
            panic!("The request has to wait");
        };

        delivery_info.ping(created.add(Duration::from_secs(1)));
        assert!(delivery_info.awaiting_response.is_some());

        delivery_info.ping(created.add(Duration::from_secs(4)));
        assert!(delivery_info.awaiting_response.is_none());

        assert!(matches!(receiver.await.unwrap(), HttpPayload::Ping));
    }
}
