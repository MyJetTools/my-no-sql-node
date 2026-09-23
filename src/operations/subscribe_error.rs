use std::fmt::Display;

#[derive(Debug)]
pub enum SubscribeError {
    /// The reader named a namespace the node does not replicate yet, and it already replicates
    /// as many as it is allowed to - every one costs a connection to the main node.
    NamespacesLimitReached { max: usize },
    /// A concurrent subscribe of the same HTTP session switched the reader to another namespace.
    NamespaceChanged,
}

impl Display for SubscribeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NamespacesLimitReached { max } => write!(
                f,
                "The node replicates {} namespaces already, which is its limit (MaxNamespaces)",
                max
            ),
            Self::NamespaceChanged => write!(
                f,
                "The namespace of the session was changed by a concurrent subscribe"
            ),
        }
    }
}
