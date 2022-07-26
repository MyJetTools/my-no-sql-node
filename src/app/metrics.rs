use prometheus::{Encoder, IntGauge, IntGaugeVec, Opts, Registry, TextEncoder};

use crate::operations::DbTableMetrics;

#[async_trait::async_trait]
pub trait UpdatePendingToSyncModel {
    async fn get_name(&self) -> Option<String>;
    fn get_pending_to_sync(&self) -> usize;
}

pub struct PrometheusMetrics {
    registry: Registry,
    partitions_amount: IntGaugeVec,
    table_size: IntGaugeVec,
    persist_amount: IntGaugeVec,
    tcp_connections_count: IntGauge,
    tcp_connections_changes: IntGaugeVec,
    fatal_errors_count: IntGauge,
    persist_delay_in_seconds: IntGaugeVec,
    pending_to_sync: IntGaugeVec,
}

const TABLE_NAME: &str = "table_name";
const TCP_METRIC: &str = "tcp_metric";

impl PrometheusMetrics {
    pub fn new() -> Self {
        let registry = Registry::new();
        let partitions_amount = create_partititions_amount_gauge();
        let table_size = create_table_size_gauge();
        let persist_amount = create_persist_amount_gauge();
        let tcp_connections_count = create_tcp_connections_count();
        let tcp_connections_changes = create_tcp_connections_changes();
        let fatal_errors_count = create_fatal_errors_count();

        let pending_to_sync = create_pending_to_sync();

        let persist_delay_in_seconds = create_persist_delay_in_seconds();

        registry
            .register(Box::new(partitions_amount.clone()))
            .unwrap();

        registry.register(Box::new(table_size.clone())).unwrap();
        registry.register(Box::new(persist_amount.clone())).unwrap();
        registry
            .register(Box::new(fatal_errors_count.clone()))
            .unwrap();

        registry
            .register(Box::new(tcp_connections_count.clone()))
            .unwrap();

        registry
            .register(Box::new(tcp_connections_changes.clone()))
            .unwrap();

        registry
            .register(Box::new(persist_delay_in_seconds.clone()))
            .unwrap();

        registry
            .register(Box::new(pending_to_sync.clone()))
            .unwrap();

        return Self {
            registry,
            partitions_amount,
            table_size,
            persist_amount,
            tcp_connections_count,
            tcp_connections_changes,
            fatal_errors_count,
            persist_delay_in_seconds,
            pending_to_sync,
        };
    }

    pub fn update_table_metrics(&self, table_name: &str, table_metrics: &DbTableMetrics) {
        let partitions_amount_value = table_metrics.partitions_amount as i64;
        self.partitions_amount
            .with_label_values(&[table_name])
            .set(partitions_amount_value);

        let table_size_value = table_metrics.table_size as i64;
        self.table_size
            .with_label_values(&[table_name])
            .set(table_size_value);

        let persist_amount_value = table_metrics.persist_amount as i64;
        self.persist_amount
            .with_label_values(&[table_name])
            .set(persist_amount_value);
    }

    pub fn update_persist_delay(&self, table_name: &str, persist_delay: i64) {
        self.persist_delay_in_seconds
            .with_label_values(&[table_name])
            .set(persist_delay);
    }

    pub async fn update_pending_to_sync<TUpdatePendingToSyncModel: UpdatePendingToSyncModel>(
        &self,
        data_reader_connection: &TUpdatePendingToSyncModel,
    ) {
        let name = data_reader_connection.get_name().await;

        if name.is_none() {
            return;
        }

        let name = name.unwrap();

        let pending_to_sync = data_reader_connection.get_pending_to_sync();

        self.pending_to_sync
            .with_label_values(&[&name])
            .set(pending_to_sync as i64);
    }

    pub async fn remove_pending_to_sync<TUpdatePendingToSyncModel: UpdatePendingToSyncModel>(
        &self,
        data_reader_connection: &TUpdatePendingToSyncModel,
    ) {
        let name = data_reader_connection.get_name().await;
        if name.is_none() {
            return;
        }

        let name = name.unwrap();
        let result = self.pending_to_sync.remove_label_values(&[&name]);

        if let Err(err) = result {
            println!(
                "Can not remove pending to sync metric for data reader {}: {:?}",
                name, err
            );
        }
    }
    pub fn mark_new_tcp_connection(&self) {
        self.tcp_connections_count.inc();
        self.tcp_connections_changes
            .with_label_values(&["connected"])
            .inc();
    }

    pub fn mark_new_tcp_disconnection(&self) {
        self.tcp_connections_count.dec();
        self.tcp_connections_changes
            .with_label_values(&["disconnected"])
            .inc();
    }

    pub fn update_fatal_errors_count(&self, value: i64) {
        self.fatal_errors_count.set(value);
    }

    pub fn build(&self) -> String {
        let mut buffer = vec![];
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        encoder.encode(&metric_families, &mut buffer).unwrap();

        return String::from_utf8(buffer).unwrap();
    }
}

fn create_partititions_amount_gauge() -> IntGaugeVec {
    let gauge_opts = Opts::new(
        format!("table_partitions_amount"),
        format!("table partitions amount"),
    );

    let lables = &[TABLE_NAME];
    IntGaugeVec::new(gauge_opts, lables).unwrap()
}

fn create_table_size_gauge() -> IntGaugeVec {
    let gauge_opts = Opts::new(format!("table_size"), format!("table size"));

    let lables = &[TABLE_NAME];
    IntGaugeVec::new(gauge_opts, lables).unwrap()
}

fn create_persist_amount_gauge() -> IntGaugeVec {
    let gauge_opts = Opts::new(format!("persist_amount"), format!("persist amount"));

    let lables = &[TABLE_NAME];
    IntGaugeVec::new(gauge_opts, lables).unwrap()
}

fn create_pending_to_sync() -> IntGaugeVec {
    let gauge_opts = Opts::new(
        format!("pending_to_send"),
        format!("pending bytes to send to reader"),
    );

    let lables = &[TABLE_NAME];
    IntGaugeVec::new(gauge_opts, lables).unwrap()
}

fn create_fatal_errors_count() -> IntGauge {
    IntGauge::new("fatal_erros_count", "Fatal errors count").unwrap()
}
fn create_tcp_connections_count() -> IntGauge {
    IntGauge::new("tcp_connections_count", "TCP Connections count").unwrap()
}

fn create_persist_delay_in_seconds() -> IntGaugeVec {
    let gauge_opts = Opts::new(
        format!("persist_delay_sec"),
        format!("Current delay of persistence operation in seconds"),
    );

    let lables = &[TABLE_NAME];
    IntGaugeVec::new(gauge_opts, lables).unwrap()
}

fn create_tcp_connections_changes() -> IntGaugeVec {
    let gauge_opts = Opts::new(format!("tcp_changes_count"), format!("Tcp Changes Count"));

    let lables = &[TCP_METRIC];
    IntGaugeVec::new(gauge_opts, lables).unwrap()
}
