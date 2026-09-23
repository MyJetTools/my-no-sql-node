use std::sync::Arc;

use my_no_sql_sdk::core::rust_extensions::{MyTimerTick, RepeatTimerIteration};

use crate::app::AppContext;

pub struct MetricsUpdater {
    app: Arc<AppContext>,
}

impl MetricsUpdater {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl MyTimerTick for MetricsUpdater {
    async fn tick(&self) -> RepeatTimerIteration {
        for namespace in self.app.namespaces.get_all().iter() {
            namespace.main_node.one_second_tick();

            self.app.metrics.update_main_node_connection(
                namespace.name.as_str(),
                namespace.main_node.is_connected(),
                namespace.main_node.get_ping_micros(),
            );

            for db_table in namespace.db.get_tables().iter() {
                let table_metrics = crate::operations::get_table_metrics(db_table.as_ref());

                self.app.metrics.update_table_metrics(
                    namespace.name.as_str(),
                    db_table.name.as_str(),
                    &table_metrics,
                );
            }
        }

        for reader in self.app.data_readers.get_all() {
            self.app.metrics.update_pending_to_sync(&reader.connection);

            reader.connection.one_sec_tick();
        }

        RepeatTimerIteration::WithInterval
    }
}
