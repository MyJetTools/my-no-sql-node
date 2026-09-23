mod request_error;
pub use request_error::*;

// The wire models are the node's own structs - see the `rest-api-shared` crate.
pub use rest_api_shared::*;

/// Name of the namespace every reader which names none works in.
pub const DEFAULT_NAMESPACE: &str = "default";
