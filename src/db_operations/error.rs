use my_no_sql_core::db_json_entity::DbEntityParseFail;

#[derive(Debug)]
pub enum DbOperationError {
    TableNotFound(String),
    TableAlreadyExists,
    RecordAlreadyExists,
    TimeStampFieldRequires,
    RecordNotFound,
    OptimisticConcurencyUpdateFails,
    TableNameValidationError(String),
    ApplicationIsNotInitializedYet,
    DbEntityParseFail(DbEntityParseFail),
}

impl DbOperationError {
    pub fn is_app_is_not_initialized(&self) -> bool {
        match self {
            DbOperationError::ApplicationIsNotInitializedYet => true,
            _ => false,
        }
    }
}
