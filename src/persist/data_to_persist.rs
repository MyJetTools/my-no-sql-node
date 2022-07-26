use std::collections::HashMap;

use rust_extensions::date_time::DateTimeAsMicroseconds;

pub enum PersistResult {
    PersistAttrs,
    PersistTable,
    PersistPartition(String),
}

impl PersistResult {
    #[cfg(test)]
    pub fn is_table(&self) -> bool {
        match self {
            PersistResult::PersistAttrs => false,
            PersistResult::PersistTable => true,
            PersistResult::PersistPartition(_) => false,
        }
    }
}

pub struct DataToPersist {
    pub persisit_whole_table: Option<DateTimeAsMicroseconds>,
    pub partitions: HashMap<String, DateTimeAsMicroseconds>,
    pub persist_attrs: bool,
}

impl DataToPersist {
    pub fn get_persist_amount(&self) -> usize {
        let mut result = if self.persist_attrs { 1 } else { 0 };
        result += self.partitions.len();

        if self.persisit_whole_table.is_some() {
            result += 1;
        };

        result
    }

    pub fn new() -> Self {
        Self {
            persisit_whole_table: None,
            partitions: HashMap::new(),
            persist_attrs: false,
        }
    }

    pub fn mark_table_to_persist(&mut self, moment: DateTimeAsMicroseconds) {
        if self.persisit_whole_table.is_none() {
            self.persisit_whole_table = Some(moment);
            return;
        }

        let persist_whole_table = self.persisit_whole_table.unwrap();

        if persist_whole_table.unix_microseconds > moment.unix_microseconds {
            self.persisit_whole_table = Some(moment)
        }
    }

    pub fn mark_partition_to_persist(
        &mut self,
        partition_key: &str,
        persist_moment: DateTimeAsMicroseconds,
    ) {
        if !self.partitions.contains_key(partition_key) {
            self.partitions
                .insert(partition_key.to_string(), persist_moment);
            return;
        }

        let moment = self.partitions.get(partition_key).unwrap().clone();

        if moment.unix_microseconds > persist_moment.unix_microseconds {
            self.partitions
                .insert(partition_key.to_string(), persist_moment);
        }
    }

    pub fn mark_persist_attrs(&mut self) {
        self.persist_attrs = true;
    }

    fn get_partition_ready_to_persist(
        &mut self,
        now: DateTimeAsMicroseconds,
        is_shutting_down: bool,
    ) -> Option<String> {
        for (key, value) in &self.partitions {
            if is_shutting_down || value.unix_microseconds <= now.unix_microseconds {
                return Some(key.to_string());
            }
        }

        None
    }

    pub fn get_next_persist_time(&self) -> Option<DateTimeAsMicroseconds> {
        if let Some(persisit_whole_table) = self.persisit_whole_table {
            return Some(persisit_whole_table);
        }

        let mut result: Option<DateTimeAsMicroseconds> = None;

        for partition_dt in self.partitions.values() {
            match result.clone() {
                Some(current_result) => {
                    if current_result.unix_microseconds > partition_dt.unix_microseconds {
                        result = Some(*partition_dt)
                    }
                }
                None => {
                    result = Some(*partition_dt);
                }
            }
        }

        result
    }

    pub fn get_what_to_persist(
        &mut self,
        now: DateTimeAsMicroseconds,
        is_shutting_down: bool,
    ) -> Option<PersistResult> {
        if let Some(persisit_whole_table) = self.persisit_whole_table {
            if persisit_whole_table.unix_microseconds <= now.unix_microseconds || is_shutting_down {
                self.persisit_whole_table = None;
                self.partitions.clear();
                return Some(PersistResult::PersistTable);
            }
        }

        if let Some(partition_key) = self.get_partition_ready_to_persist(now, is_shutting_down) {
            self.partitions.remove(partition_key.as_str());
            return Some(PersistResult::PersistPartition(partition_key));
        }

        if self.persist_attrs {
            self.persist_attrs = false;
            return Some(PersistResult::PersistAttrs);
        }
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_add_partition_with_later_date() {
        let mut data_to_persist = DataToPersist::new();

        data_to_persist.mark_partition_to_persist("test", DateTimeAsMicroseconds::new(5));

        data_to_persist.mark_partition_to_persist("test", DateTimeAsMicroseconds::new(6));

        let result = data_to_persist
            .get_what_to_persist(DateTimeAsMicroseconds::new(5), false)
            .unwrap();

        if let PersistResult::PersistPartition(table_name) = result {
            assert_eq!("test", table_name);
        } else {
            panic!("Should not be here");
        }
    }

    #[test]
    fn test_add_partition_with_table_later() {
        let mut data_to_persist = DataToPersist::new();

        data_to_persist.mark_partition_to_persist("test", DateTimeAsMicroseconds::new(5));

        data_to_persist.mark_table_to_persist(DateTimeAsMicroseconds::new(6));

        let result = data_to_persist
            .get_what_to_persist(DateTimeAsMicroseconds::new(5), false)
            .unwrap();

        if let PersistResult::PersistPartition(table_name) = result {
            assert_eq!("test", table_name);
        } else {
            panic!("Should not be here");
        }

        let result = data_to_persist.get_what_to_persist(DateTimeAsMicroseconds::new(5), false);

        assert_eq!(true, result.is_none());

        let result = data_to_persist
            .get_what_to_persist(DateTimeAsMicroseconds::new(6), false)
            .unwrap();

        assert_eq!(true, result.is_table());
    }
}
