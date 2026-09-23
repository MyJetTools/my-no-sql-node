use my_no_sql_sdk::tcp_contracts::DeleteRowTcpContract;

use crate::{
    app::AppContext,
    db_sync::{DeleteRowsEventSyncData, SyncEvent},
    namespaces::NodeNamespace,
};

pub fn delete_rows(
    app: &AppContext,
    namespace: &NodeNamespace,
    table_name: String,
    rows: Vec<DeleteRowTcpContract>,
) {
    let Some(db_table) = namespace.db.get_table(table_name.as_str()) else {
        return;
    };

    let mut sync_data = DeleteRowsEventSyncData::new(db_table.name.clone());

    {
        let mut table_data = db_table.data.write();

        for row in rows {
            if let Some((partition_key, deleted_row, _)) =
                table_data.remove_row(&row.partition_key, &row.row_key, true)
            {
                sync_data.add_deleted_row(partition_key.as_str(), deleted_row);
            }
        }
    }

    if sync_data.has_data() {
        app.dispatch(namespace.name.clone(), SyncEvent::DeleteRows(sync_data));
    }
}
