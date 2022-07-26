use my_http_server_swagger::*;
use my_no_sql_core::db::DbTable;
use serde::{Deserialize, Serialize};

use crate::db_sync::DataSynchronizationPeriod;

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

#[derive(MyHttpInput)]
pub struct CleanTableContract {
    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,
    #[http_query(name = "syncPeriod"; description = "Synchronization period"; default="Sec5")]
    pub sync_period: DataSynchronizationPeriod,
}

#[derive(MyHttpInput)]
pub struct UpdatePersistTableContract {
    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,

    #[http_query(description = "Persist table"; default="true")]
    pub persist: bool,
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

#[derive(MyHttpInput)]
pub struct CreateTableCotnract {
    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,

    #[http_query(description = "Persist table"; default="true")]
    pub persist: bool,

    #[http_query(name = "maxPartitionsAmount"; description = "Maximim partitions amount. Empty - means unlimited")]
    pub max_partitions_amount: Option<usize>,

    #[http_query(name = "syncPeriod"; description = "Synchronization period"; default="Sec5")]
    pub sync_period: DataSynchronizationPeriod,
}

#[derive(MyHttpInput)]
pub struct TableMigrationInputContract {
    #[http_query(name = "remoteUrl"; description = "Url of the remote MyNoSqlServer we are going to copy data from")]
    pub remote_url: String,

    #[http_query(name = "tableName"; description = "Table name of the current MyNoSqlServer we are going to copy data to")]
    pub table_name: String,

    #[http_query(name = "remoteTableName"; description = "Table name of the remote MyNoSqlServer we are going to copy data from")]
    pub remote_table_name: String,
}

#[derive(MyHttpInput)]
pub struct DeleteTableContract {
    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,
    #[http_header(name = "apikey"; description = "Api Key protecting the table to be deleted")]
    pub api_key: String,
}
