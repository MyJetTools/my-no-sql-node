use std::sync::Arc;

use my_no_sql_sdk::{core::db::DbNamespaceName, server::DbTable};

use crate::{
    app::AppContext,
    data_readers::DataReader,
    db_sync::{SyncEvent, TableFirstInitSyncData},
    namespaces::NodeNamespace,
};

use super::SubscribeError;

/// Subscribes a reader to a table of the namespace it works in.
///
/// A table the node holds is served straight away. Any other one is requested from the main
/// node and the reader waits for it: it is served once the table arrives - an empty one if the
/// main node does not have it, see `hold_table_as_not_found`. Waiting is what makes a reader
/// which connects before the node has connected to the main node work: nothing is answered with
/// an error, the SDK reader panics on one.
pub async fn subscribe(
    app: &Arc<AppContext>,
    data_reader: &Arc<DataReader>,
    table_name: &str,
) -> Result<(), SubscribeError> {
    let namespace_name = data_reader.get_namespace();

    let namespace = match app
        .namespaces
        .get_or_create(app, namespace_name.as_str())
        .await
    {
        Ok(namespace) => namespace,
        Err(err) => return Err(SubscribeError::NamespacesLimitReached { max: err.max }),
    };

    if let Some(db_table) = namespace.db.get_table(table_name) {
        return subscribe_to_table(app, &namespace.name, data_reader, db_table);
    }

    // Registered before the request goes out: the answer can arrive before this function returns.
    if !data_reader.add_awaiting_table(&namespace.name, table_name) {
        return Err(SubscribeError::NamespaceChanged);
    }

    namespace.main_node.request_table(table_name);

    // The table could have arrived between the look up above and the registration - nobody would
    // serve the reader then. Whoever takes the awaited table off the reader first serves it.
    if let Some(db_table) = namespace.db.get_table(table_name) {
        if data_reader.take_awaiting_table(&namespace.name, table_name) {
            return subscribe_to_table(app, &namespace.name, data_reader, db_table);
        }
    }

    Ok(())
}

pub fn unsubscribe(data_reader: &DataReader, table_name: &str) {
    data_reader.unsubscribe(table_name);
}

/// Serves the readers which wait for the table that has just been settled - arrived from the
/// main node, or held empty because the main node does not have it.
pub fn serve_awaiting_readers(
    app: &AppContext,
    namespace: &NodeNamespace,
    db_table: &Arc<DbTable>,
) {
    for data_reader in app
        .data_readers
        .take_readers_awaiting_table(&namespace.name, db_table.name.as_str())
    {
        // A reader switched to another namespace in between is not served - it does not wait
        // for this table any more.
        let _ = subscribe_to_table(app, &namespace.name, &data_reader, db_table.clone());
    }
}

fn subscribe_to_table(
    app: &AppContext,
    namespace: &DbNamespaceName,
    data_reader: &Arc<DataReader>,
    db_table: Arc<DbTable>,
) -> Result<(), SubscribeError> {
    if !data_reader.subscribe(namespace, db_table.name.as_str()) {
        return Err(SubscribeError::NamespaceChanged);
    }

    app.dispatch(
        namespace.clone(),
        SyncEvent::TableFirstInit(TableFirstInitSyncData {
            db_table,
            data_reader: data_reader.clone(),
        }),
    );

    Ok(())
}
