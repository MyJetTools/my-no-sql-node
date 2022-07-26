use app::{logs::Logs, AppContext};
use background::{
    gc_http_sessions::GcHttpSessionsTimer, gc_multipart::GcMultipart,
    metrics_updater::MetricsUpdater, sync::SyncEventLoop,
};

use my_no_sql_tcp_shared::MyNoSqlReaderTcpSerializer;
use my_tcp_sockets::TcpServer;
use rust_extensions::MyTimer;
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tcp::TcpServerEvents;
use tcp_client::TcpClientSocketCallback;

mod app;
mod tcp_client;

mod db_operations;
mod db_sync;
mod http;

mod tcp;

mod background;
mod data_readers;
mod operations;
mod settings_reader;

#[tokio::main]
async fn main() {
    let settings = settings_reader::read_settings().await;

    let settings = Arc::new(settings);

    let logs = Arc::new(Logs::new());

    let app = AppContext::new(logs.clone(), settings);

    let app = Arc::new(app);

    app.sync
        .register_event_loop(Arc::new(SyncEventLoop::new(app.clone())))
        .await;

    let mut timer_1s = MyTimer::new(Duration::from_secs(1));

    timer_1s.register_timer("MetricsUpdated", Arc::new(MetricsUpdater::new(app.clone())));

    let mut timer_10s = MyTimer::new(Duration::from_secs(10));
    timer_10s.register_timer(
        "GcHttpSessions",
        Arc::new(GcHttpSessionsTimer::new(app.clone())),
    );

    let mut timer_30s = MyTimer::new(Duration::from_secs(30));

    timer_30s.register_timer("GcMultipart", Arc::new(GcMultipart::new(app.clone())));

    timer_1s.start(app.states.clone(), app.clone());
    timer_10s.start(app.states.clone(), app.clone());
    timer_30s.start(app.states.clone(), app.clone());

    app.sync.start(app.states.clone(), app.clone()).await;

    crate::http::start_up::setup_server(&app);

    let tcp_server = TcpServer::new(
        "MyNoSqlReader".to_string(),
        SocketAddr::from(([0, 0, 0, 0], 5125)),
    );

    tcp_server
        .start(
            Arc::new(MyNoSqlReaderTcpSerializer::new),
            Arc::new(TcpServerEvents::new(app.clone())),
            app.states.clone(),
            app.clone(),
        )
        .await;

    let socket_callback = TcpClientSocketCallback::new(app.clone());

    let socket_callback = Arc::new(socket_callback);

    app.tcp_client
        .start(
            Arc::new(|| -> MyNoSqlReaderTcpSerializer { MyNoSqlReaderTcpSerializer::new() }),
            socket_callback,
            app.clone(),
        )
        .await;

    app.states.wait_until_shutdown().await;

    crate::operations::shutdown::execute(app.as_ref()).await;
}
