use std::sync::Arc;

use my_no_sql_core::db::{DbTable, DbTableAttributesSnapshot, DbTableInner};
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::app::AppContext;

pub async fn get_or_add_table(app: &Arc<AppContext>, table_name: &str) -> Arc<DbTable> {
    let db_table = app.db.get_table(table_name).await;

    if db_table.is_some() {
        return db_table.unwrap();
    }

    let now = DateTimeAsMicroseconds::now();
    let db_table_inner = DbTableInner::new(table_name.to_string(), now);
    let attr = DbTableAttributesSnapshot {
        persist: false,
        max_partitions_amount: None,
        created: now,
    };
    let db_table = DbTable::new(db_table_inner, attr);

    let db_table = Arc::new(db_table);

    {
        println!("Lazy initializing table: {}", table_name);
        let mut write_access = app.db.tables.write().await;
        write_access.insert(table_name.to_string(), db_table.clone());
    }

    let data_readers = app.data_readers.get_all().await;

    for data_reader in data_readers {
        if data_reader.has_awaiting_table(table_name).await {
            let result =
                crate::operations::data_readers::subscribe(app, data_reader, table_name).await;

            if let Err(err) = result {
                println!("Error subscribing to table: {:?}", err);
            }
        }
    }

    db_table
}
