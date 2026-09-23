use std::sync::Arc;

use my_no_sql_sdk::server::DbTable;

use crate::{db_operations::DbOperationError, namespaces::NodeNamespace};

/// A table of the namespace. The node holds only the tables its readers subscribed to, so a
/// table the main node has is still "not found" here until somebody subscribes to it.
pub fn get_table(
    namespace: &NodeNamespace,
    table_name: &str,
) -> Result<Arc<DbTable>, DbOperationError> {
    match namespace.db.get_table(table_name) {
        Some(db_table) => Ok(db_table),
        None => Err(DbOperationError::TableNotFound(table_name.to_string())),
    }
}
