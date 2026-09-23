#[derive(Debug)]
pub enum DbOperationError {
    TableNotFound(String),
    RecordNotFound,
    NamespaceNotFound(String),
    NamespaceNameValidationError(String),
}
