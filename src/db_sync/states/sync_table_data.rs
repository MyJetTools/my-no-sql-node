use my_no_sql_core::db::DbTableInner;

pub struct SyncTableData {
    pub table_name: String,
    pub persist: bool,
}

impl SyncTableData {
    pub fn new(table_data: &DbTableInner, persist: bool) -> Self {
        Self {
            table_name: table_data.name.to_string(),
            persist,
        }
    }
}
