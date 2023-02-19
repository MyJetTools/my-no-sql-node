use std::{collections::HashMap, sync::Arc};

use my_no_sql_server_core::DbTableWrapper;

pub struct DataReaderUpdatableData {
    tables: HashMap<String, Arc<DbTableWrapper>>,
}

impl DataReaderUpdatableData {
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
        }
    }

    pub fn subscribe(&mut self, db_table: Arc<DbTableWrapper>) {
        self.tables.insert(db_table.name.to_string(), db_table);
    }

    pub fn has_table(&self, table_name: &str) -> bool {
        self.tables.contains_key(table_name)
    }

    pub fn get_table_names(&self) -> Vec<String> {
        self.tables.keys().map(|id| id.to_string()).collect()
    }
}
