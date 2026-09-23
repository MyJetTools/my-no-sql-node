//! The write api of the main node, as the node exposes it: every call is forwarded to the main
//! node - see `crate::main_server_http`. Routes, deprecated routes and contracts are the main
//! node's own, so a writer connects to the node exactly as to the main node.

mod models;
mod bulk;
mod gc;
mod ping;
mod row;
mod rows;
mod tables;
mod transactions;

mod register;
pub use register::*;
