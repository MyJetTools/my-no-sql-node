use std::{collections::BTreeMap, sync::Arc};

use my_no_sql_core::{db::DbRow, db_json_entity::DbJsonEntity};

use super::DbOperationError;

pub fn as_btree_map(
    as_bytes: &[u8],
) -> Result<BTreeMap<String, Vec<Arc<DbRow>>>, DbOperationError> {
    match DbJsonEntity::parse_as_btreemap(as_bytes, &None) {
        Ok(result) => Ok(result),
        Err(err) => {
            let result = DbOperationError::DbEntityParseFail(err);
            Err(result)
        }
    }
}
