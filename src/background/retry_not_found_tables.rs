use std::sync::Arc;

use my_no_sql_sdk::core::rust_extensions::{MyTimerTick, RepeatTimerIteration};

use crate::app::AppContext;

/// Asks the main node again for the tables it said it does not have while this node holds a
/// copy of them - a table deleted while the node was not connected and created anew afterwards
/// is picked up without waiting for a reconnect.
pub struct RetryNotFoundTablesTimer {
    app: Arc<AppContext>,
}

impl RetryNotFoundTablesTimer {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl MyTimerTick for RetryNotFoundTablesTimer {
    async fn tick(&self) -> RepeatTimerIteration {
        for namespace in self.app.namespaces.get_all().iter() {
            namespace.main_node.retry_not_found_tables();
        }

        RepeatTimerIteration::WithInterval
    }
}
