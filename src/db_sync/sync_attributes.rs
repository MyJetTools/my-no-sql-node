use my_http_server::macros::*;

#[derive(Clone, Copy, MyHttpStringEnum)]
pub enum DataSynchronizationPeriod {
    #[http_enum_case(id="0"; value:"i"; description="Immediately Persist")]
    Immediately,
    #[http_enum_case(id="1"; value:"1"; description="Persist during 1 sec")]
    Sec1,
    #[http_enum_case(id="5"; value:"5";  description="Persist during 5 sec")]
    Sec5,
    #[http_enum_case(id="15"; value:"15"; description="Persist during 15 sec")]
    Sec15,
    #[http_enum_case(id="30"; value:"30"; description="Persist during 30 sec")]
    Sec30,
    #[http_enum_case(id="60"; value:"60"; description="Persist during 1 minute")]
    Min1,
    #[http_enum_case(id="6"; value:"a"; description="Sync as soon as CPU schedules task")]
    Asap,
}
