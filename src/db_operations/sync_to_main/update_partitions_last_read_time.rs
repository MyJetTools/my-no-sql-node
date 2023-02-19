use crate::{app::AppContext, background::sync_to_main_node::SyncToMainNodeEvent};

pub async fn update_partition_last_read_time(
    app: &AppContext,
    table_name: &str,
    partition: &String,
) {
    update_partitions_last_read_time(app, table_name, [partition].into_iter()).await
}

pub async fn update_partitions_last_read_time<'s, TPartitions: Iterator<Item = &'s String>>(
    app: &AppContext,
    table_name: &str,
    partitions: TPartitions,
) {
    app.sync_to_main_node_queue
        .update_partitions_last_read_time(table_name, partitions)
        .await;
    app.sync_to_main_node_events_loop
        .send(SyncToMainNodeEvent::PingToDeliver);
}
