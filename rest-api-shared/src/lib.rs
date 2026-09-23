//! Wire models of the node's HTTP api its web UI works with. The node serves and parses them,
//! the UI builds its requests from them and deserializes the answers - one definition for both
//! ends, so the two can not drift apart.
//!
//! Counters are `u64` on purpose: the UI is wasm32, where `usize` is 32 bits wide.

mod connections;
mod partitions;
mod rows;
mod status;

pub use connections::*;
pub use partitions::*;
pub use rows::*;
pub use status::*;
