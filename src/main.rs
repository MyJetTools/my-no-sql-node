use std::{net::SocketAddr, sync::Arc, time::Duration};

use app::AppContext;
use background::*;
use my_no_sql_sdk::core::rust_extensions::MyTimer;
use my_no_sql_sdk::tcp_contracts::MyNoSqlTcpSerializerFactory;
use my_tcp_sockets::TcpServer;
use tcp_server::TcpServerEvents;

mod app;
mod background;
mod data_readers;
mod db_operations;
mod db_sync;
mod http;
mod namespaces;
mod operations;
mod settings_reader;
mod tcp_client_to_main_node;
mod tcp_server;

#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[tokio::main]
async fn main() {
    let settings = settings_reader::read_settings().await;

    let app = Arc::new(AppContext::new(Arc::new(settings)));

    app.sync_to_clients
        .register_event_loop(Arc::new(SyncEventsToClients::new(app.clone())));

    app.sync_to_clients
        .start(app.states.clone(), my_logger::LOGGER.clone());

    // Every reader which names no namespace works in the default one - it is replicated from the
    // start, so the node is connected to the main node before the first reader shows up.
    if let Err(err) = app
        .namespaces
        .get_or_create(&app, my_no_sql_sdk::DEFAULT_NAMESPACE)
        .await
    {
        panic!("Can not create the default namespace. Err: {:?}", err);
    }

    let mut timer_1s = MyTimer::new(Duration::from_secs(1));
    timer_1s.register_timer("MetricsUpdater", Arc::new(MetricsUpdater::new(app.clone())));

    let mut timer_10s = MyTimer::new(Duration::from_secs(10));
    timer_10s.register_timer(
        "GcHttpSessions",
        Arc::new(GcHttpSessionsTimer::new(app.clone())),
    );

    let mut timer_30s = MyTimer::new(Duration::from_secs(30));
    timer_30s.register_timer(
        "RetryNotFoundTables",
        Arc::new(RetryNotFoundTablesTimer::new(app.clone())),
    );

    timer_1s.start(app.states.clone(), my_logger::LOGGER.clone());
    timer_10s.start(app.states.clone(), my_logger::LOGGER.clone());
    timer_30s.start(app.states.clone(), my_logger::LOGGER.clone());

    crate::http::setup_server(&app);

    let tcp_server = TcpServer::new(
        "MyNoSqlReader".to_string(),
        SocketAddr::from(([0, 0, 0, 0], app.settings.get_tcp_port())),
    );

    tcp_server
        .start(
            Arc::new(MyNoSqlTcpSerializerFactory),
            TcpServerEvents::new(app.clone()),
            app.states.clone(),
            my_logger::LOGGER.clone(),
        )
        .await;

    app.states.wait_until_shutdown().await;

    println!("Stopping the application");
}
