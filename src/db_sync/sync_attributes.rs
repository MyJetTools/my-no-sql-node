use my_http_server_swagger::*;
use std::collections::HashMap;

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

#[derive(Clone)]
pub struct ClientRequestsSourceData {
    pub locations: Vec<String>,
    pub headers: Option<HashMap<String, String>>,
}

#[derive(Clone, Copy)]
pub enum EventSource {
    SyncFromMain,
}
