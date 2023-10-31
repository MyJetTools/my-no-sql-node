use std::sync::Arc;

use my_http_server::controllers::ControllersMiddleware;

use crate::app::AppContext;

pub fn build(app: &Arc<AppContext>) -> ControllersMiddleware {
    let mut result = ControllersMiddleware::new(None, None);

    let api_controller = super::api::ApiController::new();
    result.register_get_action(Arc::new(api_controller));

    result.register_get_action(Arc::new(super::logs_controller::GetFatalErrorsAction::new(
        app.clone(),
    )));

    result.register_get_action(Arc::new(super::tables_controller::GetListAction::new(
        app.clone(),
    )));

    let get_partitions_count_action = Arc::new(
        super::tables_controller::GetPartitionsCountAction::new(app.clone()),
    );

    result.register_get_action(get_partitions_count_action);

    let get_table_size_action = Arc::new(super::tables_controller::GetTableSizeAction::new(
        app.clone(),
    ));

    result.register_get_action(get_table_size_action);

    result.register_post_action(Arc::new(super::multipart::FirstMultipartController::new(
        app.clone(),
    )));

    result.register_post_action(Arc::new(super::multipart::NextMultipartController::new(
        app.clone(),
    )));

    result.register_get_action(Arc::new(super::status_controller::StatusController::new(
        app.clone(),
    )));

    result.register_get_action(Arc::new(super::row_controller::RowCountAction::new(
        app.clone(),
    )));

    result.register_get_action(Arc::new(super::row_controller::RowAction::new(app.clone())));

    result.register_get_action(Arc::new(
        super::rows_controller::GetHighestRowAndBelowAction::new(app.clone()),
    ));

    result.register_post_action(Arc::new(
        super::rows_controller::GetSinglePartitionMultipleRowsAction::new(app.clone()),
    ));

    result.register_get_action(Arc::new(super::logs_controller::GetLogsByTableAction::new(
        app.clone(),
    )));

    result.register_get_action(Arc::new(super::logs_controller::SelectTableAction::new(
        app.clone(),
    )));

    result.register_get_action(Arc::new(
        super::logs_controller::GetLogsByProcessAction::new(app.clone()),
    ));

    result.register_get_action(Arc::new(super::logs_controller::SelectProcessAction::new()));

    result.register_get_action(Arc::new(super::logs_controller::HomeAction::new(
        app.clone(),
    )));

    result.register_get_action(Arc::new(super::home_controller::IndexAction::new(
        app.clone(),
    )));

    result.register_get_action(Arc::new(super::prometheus_controller::MetricsAction::new(
        app.clone(),
    )));

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
