use std::sync::Arc;

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{app::AppContext, background::sync_to_main_node::SyncToMainNodeEvent};

pub async fn update_partition_expiration_time(
    app: &Arc<AppContext>,
    table_name: &str,
    partition_key: &str,
    expiration_time: DateTimeAsMicroseconds,
) {
    let app = app.clone();

    app.sync_to_main_node_queue
        .update_partition_expiration_time(table_name, partition_key, expiration_time)
        .await;

    app.sync_to_main_node_events_loop
        .send(SyncToMainNodeEvent::PingToDeliver);
}
