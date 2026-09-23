use prometheus::{Encoder, IntGauge, IntGaugeVec, Opts, Registry, TextEncoder};

use crate::operations::DbTableMetrics;

pub trait UpdatePendingToSyncModel {
    fn get_name(&self) -> Option<String>;
    fn get_pending_to_sync(&self) -> usize;
}

pub struct PrometheusMetrics {
    registry: Registry,
    partitions_amount: IntGaugeVec,
    table_size: IntGaugeVec,
    tcp_connections_count: IntGauge,
    tcp_connections_changes: IntGaugeVec,
    pending_to_sync: IntGaugeVec,
    main_node_connected: IntGaugeVec,
    main_node_ping: IntGaugeVec,
}

const TABLE_NAME: &str = "table_name";
/// Namespace the metric belongs to. Always present - the default namespace reports itself as
/// "default" rather than as an absent label.
const NAMESPACE: &str = "ns";
const TCP_METRIC: &str = "tcp_metric";

impl PrometheusMetrics {
    pub fn new() -> Self {
        let registry = Registry::new();
        let partitions_amount = create_partitions_amount_gauge();
        let table_size = create_table_size_gauge();
        let tcp_connections_count = create_tcp_connections_count();
        let tcp_connections_changes = create_tcp_connections_changes();
        let pending_to_sync = create_pending_to_sync();
        let main_node_connected = create_main_node_connected();
        let main_node_ping = create_main_node_ping();

        registry
            .register(Box::new(partitions_amount.clone()))
            .unwrap();

        registry.register(Box::new(table_size.clone())).unwrap();

        registry
            .register(Box::new(tcp_connections_count.clone()))
            .unwrap();

        registry
            .register(Box::new(tcp_connections_changes.clone()))
            .unwrap();

        registry
            .register(Box::new(pending_to_sync.clone()))
            .unwrap();

        registry
            .register(Box::new(main_node_connected.clone()))
            .unwrap();

        registry.register(Box::new(main_node_ping.clone())).unwrap();

        Self {
            registry,
            partitions_amount,
            table_size,
            tcp_connections_count,
            tcp_connections_changes,
            pending_to_sync,
            main_node_connected,
            main_node_ping,
        }
    }

    pub fn update_table_metrics(
        &self,
        namespace: &str,
        table_name: &str,
        table_metrics: &DbTableMetrics,
    ) {
        self.partitions_amount
            .with_label_values(&[namespace, table_name])
            .set(table_metrics.partitions_amount as i64);

        self.table_size
            .with_label_values(&[namespace, table_name])
            .set(table_metrics.table_size as i64);
    }

    pub fn update_main_node_connection(&self, namespace: &str, connected: bool, ping_micros: i64) {
        self.main_node_connected
            .with_label_values(&[namespace])
            .set(if connected { 1 } else { 0 });

        self.main_node_ping
            .with_label_values(&[namespace])
            .set(ping_micros);
    }

    pub fn update_pending_to_sync<TUpdatePendingToSyncModel: UpdatePendingToSyncModel>(
        &self,
        data_reader_connection: &TUpdatePendingToSyncModel,
    ) {
        let Some(name) = data_reader_connection.get_name() else {
            return;
        };

        let pending_to_sync = data_reader_connection.get_pending_to_sync();

        self.pending_to_sync
            .with_label_values(&[name.as_str()])
            .set(pending_to_sync as i64);
    }

    pub fn remove_pending_to_sync<TUpdatePendingToSyncModel: UpdatePendingToSyncModel>(
        &self,
        data_reader_connection: &TUpdatePendingToSyncModel,
    ) {
        let Some(name) = data_reader_connection.get_name() else {
            return;
        };

        // Several connections of one application share the name - the label may already be
        // gone together with a sibling connection, which is not an error.
        let _ = self.pending_to_sync.remove_label_values(&[name.as_str()]);
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

    pub fn build(&self) -> String {
        let mut buffer = vec![];
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        encoder.encode(&metric_families, &mut buffer).unwrap();

        String::from_utf8(buffer).unwrap()
    }
}

fn create_partitions_amount_gauge() -> IntGaugeVec {
    let gauge_opts = Opts::new("table_partitions_amount", "table partitions amount");
    IntGaugeVec::new(gauge_opts, &[NAMESPACE, TABLE_NAME]).unwrap()
}

fn create_table_size_gauge() -> IntGaugeVec {
    let gauge_opts = Opts::new("table_size", "table size");
    IntGaugeVec::new(gauge_opts, &[NAMESPACE, TABLE_NAME]).unwrap()
}

fn create_pending_to_sync() -> IntGaugeVec {
    let gauge_opts = Opts::new("pending_to_send", "pending bytes to send to reader");

    // Labelled by the reader name, not by a table - the label name is kept as the main node has it.
    IntGaugeVec::new(gauge_opts, &[TABLE_NAME]).unwrap()
}

fn create_tcp_connections_count() -> IntGauge {
    IntGauge::new("tcp_connections_count", "TCP Connections count").unwrap()
}

fn create_tcp_connections_changes() -> IntGaugeVec {
    let gauge_opts = Opts::new("tcp_changes_count", "Tcp Changes Count");
    IntGaugeVec::new(gauge_opts, &[TCP_METRIC]).unwrap()
}

fn create_main_node_connected() -> IntGaugeVec {
    let gauge_opts = Opts::new(
        "main_node_connected",
        "1 if the node is connected to the main node for the namespace, 0 otherwise",
    );
    IntGaugeVec::new(gauge_opts, &[NAMESPACE]).unwrap()
}

fn create_main_node_ping() -> IntGaugeVec {
    let gauge_opts = Opts::new(
        "main_node_ping_microseconds",
        "Last ping round trip to the main node",
    );
    IntGaugeVec::new(gauge_opts, &[NAMESPACE]).unwrap()
}
