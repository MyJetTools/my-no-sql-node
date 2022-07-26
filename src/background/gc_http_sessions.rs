use std::sync::Arc;

use rust_extensions::{date_time::DateTimeAsMicroseconds, MyTimerTick};

use crate::app::AppContext;

pub struct GcHttpSessionsTimer {
    app: Arc<AppContext>,
}

impl GcHttpSessionsTimer {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl MyTimerTick for GcHttpSessionsTimer {
    async fn tick(&self) {
        let now = DateTimeAsMicroseconds::now();

        for data_reader in self.app.data_readers.get_all().await {
            data_reader.ping_http_servers(now).await;
        }
        if let Some(gced) = self.app.data_readers.gc_http_sessions(now).await {
            for data_reader in gced {
                self.app
                    .metrics
                    .remove_pending_to_sync(&data_reader.connection)
                    .await;
            }
        }
    }
}
