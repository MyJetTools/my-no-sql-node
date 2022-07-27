use std::sync::Arc;

use my_no_sql_tcp_shared::TcpContract;

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
            "{:?} is subscribing to the table {} which does not exist. Trying to subscribe to main node",
            data_reader.get_name().await,
            table_name
        );

        let node_connection = app.connected_to_main_node.get().await;

        if node_connection.is_none() {
            return Err(DbOperationError::NoConnectionToMainNode);
        }

        data_reader
            .add_awaiting_tables(table_name.to_string())
            .await;

        let node_connection = node_connection.unwrap();

        node_connection
            .send(TcpContract::SubscribeAsNode(table_name.to_string()))
            .await;

        return Ok(());
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
