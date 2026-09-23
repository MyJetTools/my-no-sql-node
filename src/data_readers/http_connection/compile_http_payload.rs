use my_json::json_writer::{JsonArrayWriter, JsonObjectWriter};

use crate::db_sync::{
    DeleteRowsEventSyncData, InitPartitionsSyncData, SyncEvent, UpdateRowsSyncData,
};

/// Serializes a change into the frame an HTTP reader receives from GetChanges.
pub fn compile_http_payload(sync_event: &SyncEvent) -> Vec<u8> {
    match sync_event {
        SyncEvent::TableFirstInit(sync_data) => write_init_table_result(
            sync_data.db_table.name.as_str(),
            sync_data.db_table.get_table_as_json_array(),
        ),
        SyncEvent::InitTable(sync_data) => write_init_table_result(
            sync_data.db_table.name.as_str(),
            sync_data.db_table.get_table_as_json_array(),
        ),
        SyncEvent::InitPartitions(sync_data) => write_init_partitions_result(sync_data),
        SyncEvent::UpdateRows(sync_data) => write_update_rows_result(sync_data),
        SyncEvent::DeleteRows(sync_data) => write_delete_rows_result(sync_data),
    }
}

fn write_init_table_result(table_name: &str, content: JsonArrayWriter) -> Vec<u8> {
    let header_json = JsonObjectWriter::new().write("tableName", table_name);

    let header = format!("initTable:{}", header_json.build());

    let mut result = Vec::new();
    write_pascal_string(header.as_str(), &mut result);
    write_byte_array(content.build().as_bytes(), &mut result);
    result
}

fn write_init_partitions_result(sync_data: &InitPartitionsSyncData) -> Vec<u8> {
    let header_json = JsonObjectWriter::new().write("tableName", sync_data.table_name.as_str());

    let header = format!("initPartitions:{}", header_json.build());

    let mut result = Vec::new();
    write_pascal_string(header.as_str(), &mut result);
    write_byte_array(sync_data.as_json().build().as_bytes(), &mut result);
    result
}

fn write_update_rows_result(sync_data: &UpdateRowsSyncData) -> Vec<u8> {
    let header_json = JsonObjectWriter::new().write("tableName", sync_data.table_name.as_str());

    let header = format!("updateRows:{}", header_json.build());

    let mut result = Vec::new();
    write_pascal_string(header.as_str(), &mut result);
    write_byte_array(
        sync_data
            .rows_by_partition
            .as_json_array()
            .build()
            .as_bytes(),
        &mut result,
    );
    result
}

fn write_delete_rows_result(sync_data: &DeleteRowsEventSyncData) -> Vec<u8> {
    let header_json = JsonObjectWriter::new().write("tableName", sync_data.table_name.as_str());

    let header = format!("deleteRows:{}", header_json.build());

    let mut result = Vec::new();
    write_pascal_string(header.as_str(), &mut result);
    write_byte_array(sync_data.as_json().build().as_bytes(), &mut result);
    result
}

fn write_pascal_string(src: &str, dest: &mut Vec<u8>) {
    let bytes = src.as_bytes();
    dest.push(bytes.len() as u8);
    dest.extend_from_slice(bytes)
}

fn write_byte_array(src: &[u8], dest: &mut Vec<u8>) {
    dest.extend_from_slice(&(src.len() as u32).to_le_bytes());
    dest.extend_from_slice(src);
}
