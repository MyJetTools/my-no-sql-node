use my_no_sql_sdk::{core::db::DbTableInner, server::DbTable};

use crate::{
    app::AppContext,
    db_sync::{InitTableEventSyncData, SyncEvent},
    namespaces::NodeNamespace,
};

/// The main node does not have a table this namespace asked for.
pub fn table_not_found(app: &AppContext, namespace: &NodeNamespace, table_name: String) {
    hold_table_as_not_found(app, namespace, table_name.as_str(), true);
}

/// Keeps an empty table for a table the main node could not give: the readers which asked for it
/// are subscribed to it right away - initialized with no rows, never answered with an Error
/// contract the SDK reader panics on - and the table is asked for again every 30 seconds, so its
/// rows reach them once it is created on the main node.
///
/// `clear_existing` - the node already holds a copy of a table the main node does not have any
/// more: it was deleted while the node was not connected. Its readers get an empty table, just
/// like the readers of the main node get one when a table is deleted.
pub(super) fn hold_table_as_not_found(
    app: &AppContext,
    namespace: &NodeNamespace,
    table_name: &str,
    clear_existing: bool,
) {
    let (db_table, just_created) = namespace.db.get_or_create(table_name, || {
        DbTable::new(DbTableInner::new(table_name.into()))
    });

    // Before the awaiting readers are taken: a reader which subscribes in between either finds
    // the table and subscribes to it itself, or is taken below - it can not fall in between.
    namespace.main_node.table_not_found(table_name);

    if clear_existing && !just_created {
        let had_rows = {
            let mut table_data = db_table.data.write();
            let had_rows = table_data.get_partitions_amount() > 0;
            table_data.clear_table();
            had_rows
        };

        if had_rows {
            app.dispatch(
                namespace.name.clone(),
                SyncEvent::InitTable(InitTableEventSyncData::new(db_table.clone())),
            );
        }
    }

    crate::operations::serve_awaiting_readers(app, namespace, &db_table);
}
