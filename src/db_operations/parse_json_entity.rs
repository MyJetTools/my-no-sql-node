use std::{collections::BTreeMap, sync::Arc};

use my_no_sql_core::{db::DbRow, db_json_entity::DbJsonEntity};

use super::DbOperationError;

pub fn restore_as_btree_map(
    as_bytes: &[u8],
) -> Result<BTreeMap<String, Vec<Arc<DbRow>>>, DbOperationError> {
    match DbJsonEntity::restore_as_btreemap(as_bytes) {
        Ok(result) => Ok(result),
        Err(err) => {
            let result = DbOperationError::DbEntityParseFail(err);
            Err(result)
        }
    }
}
