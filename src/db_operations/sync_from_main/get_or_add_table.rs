use std::sync::Arc;

use my_no_sql_sdk::{core::db::DbTableInner, server::DbTable};

use crate::app::AppContext;

pub async fn get_or_add_table(app: &Arc<AppContext>, table_name: &str) -> Arc<DbTable> {
    let (db_table, just_created) = app.db.get_or_create(table_name, || {
        DbTable::new(DbTableInner::new(table_name.to_string().into()))
    });

    if !just_created {
        return db_table;
    }

    println!("Lazy initializing table: {}", table_name);

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
