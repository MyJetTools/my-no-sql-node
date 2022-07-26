use my_http_server_swagger::*;
use std::collections::HashMap;

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::app::AppContext;

#[derive(Clone, Copy, MyHttpStringEnum)]
pub enum DataSynchronizationPeriod {
    #[http_enum_case(id="0"; name="i"; description="Immediately Persist")]
    Immediately,
    #[http_enum_case(id="1"; name="1"; description="Persist during 1 sec")]
    Sec1,
    #[http_enum_case(id="5"; name="5";  description="Persist during 5 sec")]
    Sec5,
    #[http_enum_case(id="15"; name="15"; description="Persist during 15 sec")]
    Sec15,
    #[http_enum_case(id="30"; name="30"; description="Persist during 30 sec")]
    Sec30,
    #[http_enum_case(id="60"; name="60"; description="Persist during 1 minute")]
    Min1,
    #[http_enum_case(id="6"; name="a"; description="Sync as soon as CPU schedules task")]
    Asap,
}

impl DataSynchronizationPeriod {
    pub fn get_sync_moment(&self) -> DateTimeAsMicroseconds {
        let mut now = DateTimeAsMicroseconds::now();

        match self {
            DataSynchronizationPeriod::Immediately => {}
            DataSynchronizationPeriod::Sec1 => now.add_seconds(1),
            DataSynchronizationPeriod::Sec5 => now.add_seconds(5),
            DataSynchronizationPeriod::Sec15 => now.add_seconds(15),
            DataSynchronizationPeriod::Sec30 => now.add_seconds(30),
            DataSynchronizationPeriod::Min1 => now.add_minutes(1),
            DataSynchronizationPeriod::Asap => {}
        }

        now
    }
}

#[derive(Clone)]
pub struct ClientRequestsSourceData {
    pub locations: Vec<String>,
    pub headers: Option<HashMap<String, String>>,
}

#[derive(Clone)]
pub enum EventSource {
    ClientRequest(ClientRequestsSourceData),
    GarbageCollector,
}

impl EventSource {
    pub fn as_gc() -> Self {
        EventSource::GarbageCollector
    }

    pub fn as_client_request(app: &AppContext) -> Self {
        let locations = vec![app.settings.location.to_string()];

        let data = ClientRequestsSourceData {
            locations,
            headers: None,
        };

        Self::ClientRequest(data)
    }
}
