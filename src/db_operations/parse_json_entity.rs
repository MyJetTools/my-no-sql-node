use std::sync::Arc;

use my_no_sql_sdk::core::{db::DbRow, db_json_entity::DbJsonEntity};

use super::DbOperationError;

pub fn restore_as_btree_map(
    as_bytes: &[u8],
) -> Result<Vec<(String, Vec<Arc<DbRow>>)>, DbOperationError> {
    match DbJsonEntity::restore_grouped_by_partition_key(as_bytes) {
        Ok(result) => Ok(result),
        Err(err) => {
            let result = DbOperationError::DbEntityParseFail(err);
            Err(result)
        }
    }
}
