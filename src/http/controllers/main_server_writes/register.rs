use std::sync::Arc;

use my_http_server::controllers::ControllersMiddleware;

use crate::app::AppContext;

pub fn register_main_server_writes(controllers: &mut ControllersMiddleware, app: &Arc<AppContext>) {
    controllers.register_post_action(Arc::new(super::ping::PingAction::new(app.clone())));
    controllers.register_post_action(Arc::new(super::bulk::BulkDeleteAction::new(app.clone())));
    controllers.register_post_action(Arc::new(super::bulk::BulkDeleteIfAction::new(app.clone())));
    controllers.register_post_action(Arc::new(super::bulk::BulkInsertOrReplaceIfNewAction::new(app.clone())));
    controllers.register_post_action(Arc::new(super::bulk::CleanAndBulkInsertAction::new(app.clone())));
    controllers.register_post_action(Arc::new(super::bulk::CleanAndBulkInsertByChunksAction::new(app.clone())));
    controllers.register_post_action(Arc::new(super::bulk::CleanAndBulkInsertByChunksCancelAction::new(app.clone())));
    controllers.register_post_action(Arc::new(super::bulk::CleanAndBulkInsertByChunksCommitAction::new(app.clone())));
    controllers.register_post_action(Arc::new(super::bulk::BulkInsertOrReplaceAction::new(app.clone())));
    controllers.register_post_action(Arc::new(super::bulk::InsertOrReplaceIfNewByChunksAction::new(app.clone())));
    controllers.register_post_action(Arc::new(super::bulk::InsertOrReplaceIfNewByChunksCancelAction::new(app.clone())));
    controllers.register_post_action(Arc::new(super::bulk::InsertOrReplaceIfNewByChunksCommitAction::new(app.clone())));
    controllers.register_post_action(Arc::new(super::gc::CleanAndKeepMaxPartitionsAmountAction::new(app.clone())));
    controllers.register_post_action(Arc::new(super::gc::CleanPartitionAndKepMaxRecordsControllerAction::new(app.clone())));
    controllers.register_delete_action(Arc::new(super::row::DeleteRowAction::new(app.clone())));
    controllers.register_delete_action(Arc::new(super::row::DeleteRowIfAction::new(app.clone())));
    controllers.register_post_action(Arc::new(super::row::InsertRowAction::new(app.clone())));
    controllers.register_post_action(Arc::new(super::row::InsertOrReplaceAction::new(app.clone())));
    controllers.register_post_action(Arc::new(super::row::InsertOrReplaceIfNewAction::new(app.clone())));
    controllers.register_put_action(Arc::new(super::row::ReplaceRowAction::new(app.clone())));
    controllers.register_delete_action(Arc::new(super::rows::DeletePartitionsAction::new(app.clone())));
    controllers.register_put_action(Arc::new(super::tables::CleanTableAction::new(app.clone())));
    controllers.register_post_action(Arc::new(super::tables::CreateIfNotExistsAction::new(app.clone())));
    controllers.register_post_action(Arc::new(super::tables::CreateTableAction::new(app.clone())));
    controllers.register_delete_action(Arc::new(super::tables::DeleteTableAction::new(app.clone())));
    controllers.register_post_action(Arc::new(super::tables::UpdateCompressedAction::new(app.clone())));
    controllers.register_post_action(Arc::new(super::tables::UpdatePersistAction::new(app.clone())));
    controllers.register_post_action(Arc::new(super::transactions::AppendTransactionAction::new(app.clone())));
    controllers.register_post_action(Arc::new(super::transactions::CancelTransactionAction::new(app.clone())));
    controllers.register_post_action(Arc::new(super::transactions::CommitTransactionAction::new(app.clone())));
    controllers.register_post_action(Arc::new(super::transactions::StartTransactionAction::new(app.clone())));
}
