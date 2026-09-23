use std::sync::Arc;

use arc_swap::ArcSwap;
use parking_lot::Mutex;

use crate::{app::AppContext, tcp_client_to_main_node::TcpClientSocketCallback};

use super::NodeNamespace;

/// Every namespace this node replicates.
///
/// A namespace comes to life when the first reader subscribes in it - together with its own
/// connection to the main node. The default namespace is created at start up, so the node is
/// connected to the main node from the very beginning.
///
/// Looked up on every reader packet and every HTTP request, created a handful of times per
/// process life - hence copy-on-write behind an `ArcSwap` over a plain `Vec`: a deployment runs
/// a few namespaces at most, and a linear scan beats hashing at that size.
///
/// A namespace lives until the node restarts, and each one costs a connection to the main node -
/// so their amount is capped: a reader naming yet another one past the cap is refused.
pub struct NodeNamespaces {
    items: ArcSwap<Vec<Arc<NodeNamespace>>>,
    /// Serializes the writers - the swap is atomic, the read-modify-write around it is not.
    write_lock: Mutex<()>,
    max_namespaces: usize,
}

#[derive(Debug)]
pub struct NamespacesLimitReached {
    pub max: usize,
}

impl NodeNamespaces {
    pub fn new(max_namespaces: usize) -> Self {
        Self {
            items: ArcSwap::from_pointee(Vec::new()),
            write_lock: Mutex::new(()),
            max_namespaces,
        }
    }

    pub fn get(&self, name: &str) -> Option<Arc<NodeNamespace>> {
        self.items
            .load()
            .iter()
            .find(|itm| itm.name.as_str() == name)
            .cloned()
    }

    pub fn get_all(&self) -> Arc<Vec<Arc<NodeNamespace>>> {
        self.items.load_full()
    }

    /// Returns the namespace, creating it and starting its connection to the main node if it is
    /// mentioned for the first time. The name must be validated by the caller.
    pub async fn get_or_create(
        &self,
        app: &Arc<AppContext>,
        name: &str,
    ) -> Result<Arc<NodeNamespace>, NamespacesLimitReached> {
        if let Some(namespace) = self.get(name) {
            return Ok(namespace);
        }

        let namespace = {
            let _write_lock = self.write_lock.lock();

            // Somebody could have created it while we were waiting for the lock.
            if let Some(namespace) = self.get(name) {
                return Ok(namespace);
            }

            if self.items.load().len() >= self.max_namespaces {
                return Err(NamespacesLimitReached {
                    max: self.max_namespaces,
                });
            }

            let namespace = Arc::new(NodeNamespace::new(
                name.into(),
                app.settings.as_ref(),
                &app.states,
            ));

            let mut items = self.items.load().as_ref().clone();
            items.push(namespace.clone());
            self.items.store(Arc::new(items));

            namespace
        };

        // Started outside of the lock by the one who created it. A reader which finds the
        // namespace in between just registers what it waits for - it is requested as soon as the
        // connection is up.
        namespace
            .main_node
            .start(
                TcpClientSocketCallback::new(app.clone(), namespace.clone()),
                app.states.clone(),
            )
            .await;

        Ok(namespace)
    }
}
