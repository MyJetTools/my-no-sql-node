use crate::{app::AppContext, background::sync_to_main_node::SyncToMainNodeEvent};

pub async fn update_row_keys_last_read_access_time<'s, TRowKeys: Iterator<Item = &'s String>>(
    app: &AppContext,
    table_name: &str,
    partition_key: &str,
    db_rows: TRowKeys,
) {
    app.sync_to_main_node_queue
        .update_rows_last_read_time(table_name, partition_key, db_rows)
        .await;

    app.sync_to_main_node_events_loop
        .send(SyncToMainNodeEvent::PingToDeliver);
}
