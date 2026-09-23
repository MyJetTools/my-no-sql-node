//! Calls of the node's HTTP api. Every request is built from a model of `rest-api-shared` - the
//! struct the node parses it with - and every answer is decoded by one of the helpers below.

use flurl::{EmptyRequestModel, FlUrl, FlUrlError, FlUrlResponse, HttpVerb};
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::models::*;

fn is_success(status: u16) -> bool {
    (200..300).contains(&status)
}

async fn read_error_body(response: &mut FlUrlResponse) -> RequestError {
    let status = response.get_status_code();

    let body = response
        .get_body_as_str()
        .await
        .map(|body| body.to_string())
        .unwrap_or_else(|err| err.to_string());

    // The body goes into the message only when it is a short plain text - a proxy in front of the
    // node answers with whole html pages.
    let body_trimmed = body.trim();
    let message = if !body_trimmed.is_empty()
        && body_trimmed.len() <= 200
        && !body_trimmed.starts_with('<')
    {
        format!("The node answered {}: {}", status, body_trimmed)
    } else {
        format!("The node answered {}", status)
    };

    RequestError {
        message,
        details: body,
    }
}

/// 2xx - the body deserialized into `T`; any other status - `Err` carrying the body.
async fn handle_http_response<T: DeserializeOwned>(
    response: Result<FlUrlResponse, FlUrlError>,
) -> Result<T, RequestError> {
    let mut response = response?;

    if is_success(response.get_status_code()) {
        return Ok(response.get_json().await?);
    }

    Err(read_error_body(&mut response).await)
}

/// The namespace the UI works in, as the `ns` header carries it: `None` - the default one, and
/// no header is sent at all.
fn namespace() -> Option<String> {
    crate::storage::load_namespace()
}

/// Node-wide - every namespace, reader and table. The UI's tables and namespaces come from here.
pub async fn get_status() -> Result<StatusModel, RequestError> {
    let response = FlUrl::new("/api/Status")
        .execute_request(HttpVerb::Get, EmptyRequestModel)
        .await;

    handle_http_response(response).await
}

/// Node-wide - every reader, every connection to the main node.
pub async fn get_connections() -> Result<ConnectionsContract, RequestError> {
    let response = FlUrl::new("/api/Connections")
        .execute_request(HttpVerb::Get, EmptyRequestModel)
        .await;

    handle_http_response(response).await
}

/// The partitions of the table whose key contains `filter`, with their records count and data
/// size, in the table's partition order - `limit` of them at most.
pub async fn get_partition_details(
    table_name: &str,
    filter: &str,
    limit: usize,
) -> Result<PartitionsDetailsContract, RequestError> {
    let filter = filter.trim();

    let response = FlUrl::new("/api/Partitions/Details")
        .execute_request(
            HttpVerb::Get,
            GetPartitionsDetailsContract {
                namespace: namespace(),
                table_name: table_name.to_string(),
                filter: if filter.is_empty() {
                    None
                } else {
                    Some(filter.to_string())
                },
                limit: Some(limit),
            },
        )
        .await;

    handle_http_response(response).await
}

/// Rows of the partition. Asks for no statistics update: looking at the data in the UI must not
/// move an expiration or a last read time the node forwards to the main node.
pub async fn get_rows(table_name: &str, partition_key: &str) -> Result<Vec<Value>, RequestError> {
    let response = FlUrl::new("/api/Row")
        .execute_request(
            HttpVerb::Get,
            GetRowInputModel {
                namespace: namespace(),
                table_name: table_name.to_string(),
                partition_key: Some(partition_key.to_string()),
                row_key: None,
                limit: None,
                skip: None,
                update_partition_last_read_access_time: None,
                set_partition_expiration_time: None,
                update_db_rows_last_read_access_time: None,
                set_db_rows_expiration_time: None,
            },
        )
        .await;

    handle_http_response(response).await
}
