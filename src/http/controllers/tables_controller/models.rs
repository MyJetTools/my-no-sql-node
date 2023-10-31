use my_http_server::macros::*;
use serde::{Deserialize, Serialize};

use my_no_sql_server_core::DbTableWrapper;

#[derive(MyHttpInput)]
pub struct GetTableSizeContract {
    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,
}

#[derive(MyHttpInput)]
pub struct GetPartitionsAmountContract {
    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,
}

#[derive(Deserialize, Serialize, MyHttpObjectStructure)]
pub struct TableContract {
    pub name: String,
}

impl TableContract {
    pub async fn from_table_wrapper(table_wrapper: &DbTableWrapper) -> Self {
        Self {
            name: table_wrapper.name.to_string(),
        }
    }
}
