use std::sync::Arc;

use my_no_sql_sdk::server::DbTable;

use crate::{app::AppContext, db_operations::DbOperationError};

pub async fn get(app: &AppContext, table_name: &str) -> Result<Arc<DbTable>, DbOperationError> {
    super::super::check_app_states(app)?;

    let get_table_result = app.db.get_table(table_name).await;

    match get_table_result {
        Some(db_table) => Ok(db_table),
        None => Err(DbOperationError::TableNotFound(table_name.to_string())),
    }
}
