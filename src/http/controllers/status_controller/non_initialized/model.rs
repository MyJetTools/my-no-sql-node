use crate::app::AppContext;
use my_http_server_swagger::*;
use rust_extensions::date_time::DateTimeAsMicroseconds;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct NonInitializedModel {
    #[serde(rename = "tablesTotal")]
    tables_total: usize,
    #[serde(rename = "tablesLoaded")]
    tables_loaded: usize,
    #[serde(rename = "filesTotal")]
    files_total: usize,
    #[serde(rename = "filesLoaded")]
    files_loaded: usize,
    #[serde(rename = "initializingSeconds")]
    loading_time: i64,
}

impl NonInitializedModel {
    pub async fn new(app: &AppContext) -> Self {
        let now = DateTimeAsMicroseconds::now();

        let snapshot = app.init_state.get_snapshot().await;

        Self {
            tables_total: snapshot.tables_total,
            files_total: snapshot.files_total,
            files_loaded: snapshot.files_loaded,
            tables_loaded: snapshot.tables_loaded,
            loading_time: now.seconds_before(app.created),
        }
    }
}
