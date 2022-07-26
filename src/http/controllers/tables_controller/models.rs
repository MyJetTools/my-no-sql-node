use my_http_server_swagger::*;
use my_no_sql_core::db::DbTable;
use serde::{Deserialize, Serialize};

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
    pub persist: bool,
    #[serde(rename = "maxPartitionsAmount")]
    pub max_partitions_amount: Option<usize>,
}

impl Into<TableContract> for &DbTable {
    fn into(self) -> TableContract {
        let table_snapshot = self.attributes.get_snapshot();
        TableContract {
            name: self.name.to_string(),
            persist: table_snapshot.persist,
            max_partitions_amount: table_snapshot.max_partitions_amount,
        }
    }
}
