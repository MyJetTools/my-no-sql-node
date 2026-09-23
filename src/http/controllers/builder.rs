use std::sync::Arc;

use my_http_server::controllers::ControllersMiddleware;

use crate::app::AppContext;

pub fn build(app: &Arc<AppContext>) -> ControllersMiddleware {
    let mut result = ControllersMiddleware::new(None, None);

    // Monitoring
    result.register_get_action(Arc::new(super::api::IsAliveAction));
    result.register_get_action(Arc::new(super::status_controller::StatusAction::new(
        app.clone(),
    )));
    result.register_get_action(Arc::new(super::prometheus_controller::MetricsAction::new(
        app.clone(),
    )));
    result.register_get_action(Arc::new(
        super::connections_controller::GetConnectionsAction::new(app.clone()),
    ));

    // Namespaces and tables
    result.register_get_action(Arc::new(
        super::namespaces_controller::GetNamespacesListAction::new(app.clone()),
    ));
    result.register_get_action(Arc::new(super::tables_controller::GetListAction::new(
        app.clone(),
    )));
    result.register_get_action(Arc::new(
        super::tables_controller::GetTablePartitionsCountAction::new(app.clone()),
    ));
    result.register_get_action(Arc::new(super::tables_controller::GetTableSizeAction::new(
        app.clone(),
    )));

    // Reads
    result.register_get_action(Arc::new(super::row_controller::RowCountAction::new(
        app.clone(),
    )));
    result.register_get_action(Arc::new(super::row_controller::GetRowsAction::new(
        app.clone(),
    )));
    result.register_get_action(Arc::new(
        super::rows_controller::GetHighestRowAndBelowAction::new(app.clone()),
    ));
    result.register_post_action(Arc::new(
        super::rows_controller::GetSinglePartitionMultipleRowsAction::new(app.clone()),
    ));
    result.register_get_action(Arc::new(
        super::partitions_controller::GetPartitionsAction::new(app.clone()),
    ));
    result.register_get_action(Arc::new(
        super::partitions_controller::GetPartitionsCountAction::new(app.clone()),
    ));
    result.register_get_action(Arc::new(
        super::partitions_controller::GetPartitionsDetailsAction::new(app.clone()),
    ));

    // HTTP readers
    result.register_post_action(Arc::new(
        super::data_reader_controller::GreetingAction::new(app.clone()),
    ));
    result.register_post_action(Arc::new(
        super::data_reader_controller::SubscribeAction::new(app.clone()),
    ));
    result.register_post_action(Arc::new(
        super::data_reader_controller::GetChangesAction::new(app.clone()),
    ));
    result.register_post_action(Arc::new(super::data_reader_controller::PingAction::new(
        app.clone(),
    )));

    result
}
