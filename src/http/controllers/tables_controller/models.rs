use my_http_server::macros::*;
use serde::{Deserialize, Serialize};

#[derive(MyHttpInput)]
pub struct GetTablesListContract {
    #[http_header(name = "ns"; description = "Namespace to work in. Empty or absent means the default namespace")]
    pub namespace: Option<String>,
}

#[derive(MyHttpInput)]
pub struct GetTableSizeContract {
    #[http_header(name = "ns"; description = "Namespace to work in. Empty or absent means the default namespace")]
    pub namespace: Option<String>,

    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,
}

#[derive(MyHttpInput)]
pub struct GetPartitionsAmountContract {
    #[http_header(name = "ns"; description = "Namespace to work in. Empty or absent means the default namespace")]
    pub namespace: Option<String>,

    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,
}

#[derive(Deserialize, Serialize, MyHttpObjectStructure)]
pub struct TableContract {
    pub name: String,
}
