use std::{net::SocketAddr, sync::Arc};

use my_http_server::{
    controllers::swagger::SwaggerMiddleware, MyHttpServer, StaticFilesMiddleware,
};

use crate::app::AppContext;

pub fn setup_server(app: &Arc<AppContext>) {
    let mut http_server = MyHttpServer::new(SocketAddr::from((
        [0, 0, 0, 0],
        app.settings.get_http_port(),
    )));

    let controllers = Arc::new(super::controllers::build(app));

    let swagger_middleware = SwaggerMiddleware::new(
        controllers.clone(),
        "MyNoSqlNode".to_string(),
        crate::app::APP_VERSION.to_string(),
    );

    http_server.add_middleware(Arc::new(swagger_middleware));
    http_server.add_middleware(controllers);
    http_server.add_middleware(Arc::new(super::UiRoutesMiddleware));
    http_server.add_middleware(Arc::new(super::StaticFilesGuard));
    // The files are read once and kept gzipped in memory - the wasm bundle is a megabyte. ETag
    // revalidation, since `assets/app.css` keeps its name from build to build: a plainly cached
    // copy would outlive an upgrade of the UI.
    http_server.add_middleware(Arc::new(
        StaticFilesMiddleware::new()
            .enable_files_caching()
            .with_etag(),
    ));

    // One listener for both protocols - h1 and h2c (prior knowledge) clients alike, the way the
    // main node accepts them.
    http_server.start_auto(app.states.clone(), my_logger::LOGGER.clone());
}
