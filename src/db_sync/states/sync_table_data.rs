use my_no_sql_core::db::DbTableInner;

pub struct SyncTableData {
    pub table_name: String,
}

impl SyncTableData {
    pub fn new(table_data: &DbTableInner) -> Self {
        Self {
            table_name: table_data.name.to_string(),
        }
    }
}
