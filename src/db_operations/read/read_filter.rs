use std::sync::Arc;

use my_no_sql_core::db::DbRow;
use rust_extensions::lazy::LazyVec;

/*
pub struct DbRowsFilter<'s, TIter: Iterator<Item = &'s Arc<DbRow>>> {
    pub iterator: TIter,
    limit: Option<usize>,
    skip: Option<usize>,
    skipped: bool,
    yielded: usize,
}

impl<'s, TIter: Iterator<Item = &'s Arc<DbRow>>> DbRowsFilter<'s, TIter> {
    pub fn new(iterator: TIter, limit: Option<usize>, skip: Option<usize>) -> Self {
        Self {
            iterator,
            limit,
            skip,
            skipped: false,
            yielded: 0,
        }
    }
}

impl<'s, TIter: Iterator<Item = &'s Arc<DbRow>>> Iterator for DbRowsFilter<'s, TIter> {
    type Item = &'s Arc<DbRow>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(skip) = self.skip {
            if !self.skipped {
                for _ in 0..skip {
                    self.iterator.next()?;
                }

                self.skipped = true;
            }
        }

        if let Some(limit) = self.limit {
            if self.yielded >= limit {
                return None;
            }
        }

        let result = self.iterator.next()?;

        self.yielded += 1;
        return Some(result);
    }
}
 */

pub fn filter_it<'s, TIter: Iterator<Item = &'s Arc<DbRow>>>(
    iterator: TIter,
    limit: Option<usize>,
    skip: Option<usize>,
) -> Option<Vec<&'s Arc<DbRow>>> {
    let mut result = if let Some(limit) = limit {
        LazyVec::with_capacity(limit)
    } else {
        LazyVec::new()
    };

    let mut no = 0;
    let mut added = 0;

    for db_row in iterator {
        if let Some(skip) = skip {
            if no < skip {
                no += 1;
                continue;
            }
        }

        result.add(db_row);
        added += 1;

        if let Some(limit) = limit {
            if added >= limit {
                break;
            }
        }

        no += 1;
        //json_array_writer.write_raw_element(&db_row.data);
        //crate::db_operations::sync_to_main::update_row_last_read_access_time(app, db_row);
    }

    result.get_result()
    //json_array_writer.build()
}
