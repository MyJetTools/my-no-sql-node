use std::sync::Arc;

use crate::{
    app::AppContext,
    data_readers::DataReader,
    db_operations::DbOperationError,
    db_sync::{states::TableFirstInitSyncData, SyncEvent},
};

pub async fn subscribe(
    app: &AppContext,
    data_reader: Arc<DataReader>,
    table_name: &str,
) -> Result<(), DbOperationError> {
    let table = app.db.get_table(table_name).await;

    if table.is_none() {
        println!(
            "{:?} is subscribing to the table {} which does not exist",
            data_reader.get_name().await,
            table_name
        );

        return Err(DbOperationError::TableNotFound(table_name.to_string()));
    }

    let db_table = table.unwrap();

    data_reader.subscribe(db_table.clone()).await;

    crate::operations::sync::dispatch(
        app,
        SyncEvent::TableFirstInit(TableFirstInitSyncData {
            db_table,
            data_reader,
        }),
    );

    Ok(())
}
