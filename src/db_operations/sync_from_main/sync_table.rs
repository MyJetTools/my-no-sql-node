use my_no_sql_sdk::{
    core::{db::DbTableInner, db_json_entity::DbJsonEntity},
    server::DbTable,
};

use crate::{
    app::AppContext,
    db_sync::{InitTableEventSyncData, SyncEvent},
    namespaces::NodeNamespace,
};

/// The whole table - an answer to a subscription, or the main node replacing the table.
pub fn sync_table(app: &AppContext, namespace: &NodeNamespace, table_name: String, data: Vec<u8>) {
    let entities = match DbJsonEntity::restore_grouped_by_partition_key(data.as_slice()) {
        Ok(entities) => entities,
        Err(err) => {
            super::report_broken_payload("InitTable", namespace, table_name.as_str(), &err);
            // The readers waiting for the table are not left hanging, and the table is asked for
            // again later. The rows the node may hold already are kept - better than none.
            super::hold_table_as_not_found(app, namespace, table_name.as_str(), false);
            return;
        }
    };

    let (db_table, _) = namespace.db.get_or_create(table_name.as_str(), || {
        DbTable::new(DbTableInner::new(table_name.as_str().into()))
    });

    {
        let mut table_data = db_table.data.write();

        table_data.clear_table();

        for (partition_key, db_rows) in entities {
            table_data.bulk_insert_or_replace(&partition_key, &db_rows);
        }
    }

    namespace.main_node.table_replicated(table_name.as_str());

    app.dispatch(
        namespace.name.clone(),
        SyncEvent::InitTable(InitTableEventSyncData::new(db_table.clone())),
    );

    crate::operations::serve_awaiting_readers(app, namespace, &db_table);
}
