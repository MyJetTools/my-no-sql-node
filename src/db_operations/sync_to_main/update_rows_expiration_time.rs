use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{app::AppContext, background::sync_to_main_node::SyncToMainNodeEvent};

pub async fn update_rows_expiration_time<'s, TRowKeys: Iterator<Item = &'s String>>(
    app: &AppContext,
    table_name: &str,
    partition_key: &str,
    row_keys: TRowKeys,
    update_expiration_time: DateTimeAsMicroseconds,
) {
    app.sync_to_main_node_queue
        .update_rows_expiration_time(table_name, partition_key, row_keys, update_expiration_time)
        .await;
    app.sync_to_main_node_events_loop
        .send(SyncToMainNodeEvent::PingToDeliver);
}
