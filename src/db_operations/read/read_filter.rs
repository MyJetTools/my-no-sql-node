use std::sync::Arc;

use my_no_sql_sdk::core::db::DbRow;
use my_no_sql_sdk::core::my_json::json_writer::JsonArrayWriter;

pub fn filter_and_compile_json<'s>(
    iterator: impl Iterator<Item = &'s Arc<DbRow>>,
    limit: Option<usize>,
    skip: Option<usize>,
) -> (JsonArrayWriter, Vec<Arc<DbRow>>) {
    let mut result = JsonArrayWriter::new();

    let mut db_rows = Vec::new();

    let mut no = 0;
    let mut added = 0;

    for db_row in iterator {
        if let Some(skip) = skip {
            if no < skip {
                no += 1;
                continue;
            }
        }

        db_rows.push(db_row.clone());

        result = result.write(db_row.as_ref());
        added += 1;

        if let Some(limit) = limit {
            if added >= limit {
                break;
            }
        }

        no += 1;
    }

    (result, db_rows)
}

pub fn filter_it<'s, TItem>(
    iterator: impl Iterator<Item = &'s TItem>,
    limit: Option<usize>,
    skip: Option<usize>,
) -> Vec<&'s TItem> {
    let mut result = if let Some(limit) = limit {
        Vec::with_capacity(limit)
    } else {
        Vec::new()
    };

    let mut no = 0;
    let mut added = 0;

    for item in iterator {
        if let Some(skip) = skip {
            if no < skip {
                no += 1;
                continue;
            }
        }

        result.push(item);
        added += 1;

        if let Some(limit) = limit {
            if added >= limit {
                break;
            }
        }

        no += 1;
    }

    result
}
