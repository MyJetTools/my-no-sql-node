use std::fmt;

/// What every api call of the UI fails with. `message` is short enough to be shown;
/// `details` is what went wrong, for the browser console.
pub struct RequestError {
    pub message: String,
    pub details: String,
}

impl fmt::Display for RequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<flurl::FlUrlError> for RequestError {
    fn from(err: flurl::FlUrlError) -> Self {
        Self {
            message: "The node does not answer".to_string(),
            details: format!("{:?}", err),
        }
    }
}

impl From<serde_json::Error> for RequestError {
    fn from(err: serde_json::Error) -> Self {
        Self {
            message: "The node answered with something the UI does not understand".to_string(),
            details: err.to_string(),
        }
    }
}
