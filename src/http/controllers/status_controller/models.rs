use crate::app::AppContext;
use my_http_server::macros::*;
use rust_extensions::date_time::DateTimeAsMicroseconds;
use serde::{Deserialize, Serialize};

use super::status_bar_model::StatusBarModel;

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct TableModel {
    pub name: String,
    #[serde(rename = "partitionsCount")]
    pub partitions_count: usize,
    #[serde(rename = "dataSize")]
    pub data_size: usize,
    #[serde(rename = "recordsAmount")]
    pub records_amount: usize,
}

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct ReaderModel {
    id: String,
    pub name: String,
    pub ip: String,
    pub tables: Vec<String>,
    #[serde(rename = "lastIncomingTime")]
    pub last_incoming_time: String,
    #[serde(rename = "connectedTime")]
    pub connected_time: String,
    #[serde(rename = "pendingToSend")]
    pub pending_to_send: usize,
    #[serde(rename = "sentPerSecond")]
    pub sent_per_second: Vec<usize>,
    #[serde(rename = "isNode")]
    pub is_node: bool,
}
#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct StatusModel {
    #[serde(rename = "notInitialized")]
    not_initialized: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    initialized: Option<InitializedModel>,
    #[serde(rename = "statusBar")]
    status_bar: StatusBarModel,
}

impl StatusModel {
    pub async fn new(app: &AppContext) -> Self {
        let (readers, tcp, http) = get_readers(app).await;
        let tables = app.db.get_tables().await;

        let mut tables_model = Vec::new();

        for table in &tables {
            let metrics = crate::operations::get_table_metrics(table.as_ref()).await;

            let table_model = TableModel {
                name: table.name.clone(),
                partitions_count: metrics.partitions_amount,
                data_size: metrics.table_size,
                records_amount: metrics.records_amount,
            };

            tables_model.push(table_model);
        }

        if app.states.is_initialized() {
            return Self {
                not_initialized: true,
                initialized: Some(InitializedModel::new(readers, tables_model)),
                status_bar: StatusBarModel::new(app, tcp, http, tables.len()),
            };
        }

        return Self {
            not_initialized: false,
            initialized: None,
            status_bar: StatusBarModel::new(app, tcp, http, tables.len()),
        };
    }
}
#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct InitializedModel {
    pub readers: Vec<ReaderModel>,
    pub tables: Vec<TableModel>,
}

impl InitializedModel {
    pub fn new(readers: Vec<ReaderModel>, tables: Vec<TableModel>) -> Self {
        Self { readers, tables }
    }
}

async fn get_readers(app: &AppContext) -> (Vec<ReaderModel>, usize, usize) {
    let mut result = Vec::new();
    let now = DateTimeAsMicroseconds::now();

    let mut tcp_count = 0;
    let mut http_count = 0;

    for data_reader in app.data_readers.get_all().await {
        match &data_reader.connection {
            crate::data_readers::DataReaderConnection::Tcp(_) => tcp_count += 1,
            crate::data_readers::DataReaderConnection::Http(_) => http_count += 1,
        }

        let metrics = data_reader.get_metrics().await;

        if let Some(name) = metrics.name {
            result.push(ReaderModel {
                connected_time: metrics.connected.to_rfc3339(),
                last_incoming_time: format!(
                    "{:?}",
                    now.duration_since(metrics.last_incoming_moment)
                        .as_positive_or_zero()
                ),
                id: metrics.session_id,
                ip: metrics.ip,
                name,
                tables: metrics.tables,
                pending_to_send: metrics.pending_to_send,
                sent_per_second: data_reader.get_sent_per_second().await,
                is_node: data_reader.is_node(),
            });
        }
    }

    (result, tcp_count, http_count)
}
