use std::sync::Arc;

use rust_extensions::MyTimerTick;

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
    async fn tick(&self) {
        let tables = self.app.db.get_tables().await;

        for db_table in tables {
            let table_metrics = crate::operations::get_table_metrics(db_table.as_ref()).await;

            self.app
                .metrics
                .update_table_metrics(db_table.name.as_str(), &table_metrics);
        }

        let fatal_errors_count = self.app.logs.get_fatal_errors_amount();

        self.app
            .metrics
            .update_fatal_errors_count(fatal_errors_count);

        for reader in self.app.data_readers.get_all().await {
            self.app
                .metrics
                .update_pending_to_sync(&reader.connection)
                .await;

            reader.connection.one_sec_tick().await;
        }
    }
}
