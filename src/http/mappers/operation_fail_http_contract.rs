use my_http_server::macros::*;
use serde::{Deserialize, Serialize};

/// The ids are the main node's ones - clients tell the reasons apart the same way whichever of
/// the two they talk to.
#[derive(Debug, MyHttpIntegerEnum)]
pub enum OperationFailReason {
    #[http_enum_case(id = -2; description = "Table not found")]
    TableNotFound,
    #[http_enum_case(id = -4; description = "Entity required field is missing")]
    RequiredEntityFieldIsMissing,
    #[http_enum_case(id = -6; description = "Namespace not found")]
    NamespaceNotFound,
}

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct OperationFailHttpContract {
    pub reason: OperationFailReason,
    pub message: String,
}
