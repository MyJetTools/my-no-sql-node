use std::{collections::HashMap, time::Duration};

use rust_extensions::date_time::DateTimeAsMicroseconds;
use tokio::sync::Mutex;

use super::data_to_persist::{DataToPersist, PersistResult};

pub struct PersistMarkersByTableInner {
    pub data_to_persist: DataToPersist,
    pub persist_duration: Vec<usize>,
    pub last_persist_time: Option<DateTimeAsMicroseconds>,
}

impl PersistMarkersByTableInner {
    pub fn new() -> Self {
        Self {
            data_to_persist: DataToPersist::new(),
            persist_duration: Vec::new(),
            last_persist_time: None,
        }
    }

    pub fn add_persist_duration(&mut self, dur: Duration) {
        self.persist_duration.push(dur.as_micros() as usize);

        while self.persist_duration.len() > 120 {
            self.persist_duration.remove(0);
        }

        self.last_persist_time = DateTimeAsMicroseconds::now().into();
    }
}

pub struct PersistMetrics {
    pub last_persist_time: Option<DateTimeAsMicroseconds>,
    pub next_persist_time: Option<DateTimeAsMicroseconds>,
    pub persist_amount: usize,
    pub last_persist_duration: Vec<usize>,
}

pub struct PersistMarkersByTable {
    persist_by_table: Mutex<HashMap<String, PersistMarkersByTableInner>>,
}

impl PersistMarkersByTable {
    pub fn new() -> Self {
        Self {
            persist_by_table: Mutex::new(HashMap::new()),
        }
    }

    pub async fn persist_partition(
        &self,
        table_name: &str,
        partition_key: &str,
        sync_moment: DateTimeAsMicroseconds,
    ) {
        let mut write_access = self.persist_by_table.lock().await;

        if !write_access.contains_key(table_name) {
            write_access.insert(table_name.to_string(), PersistMarkersByTableInner::new());
        }

        write_access
            .get_mut(table_name)
            .unwrap()
            .data_to_persist
            .mark_partition_to_persist(partition_key, sync_moment);
    }

    pub async fn persist_table(&self, table_name: &str, sync_moment: DateTimeAsMicroseconds) {
        let mut write_access = self.persist_by_table.lock().await;

        if !write_access.contains_key(table_name) {
            write_access.insert(table_name.to_string(), PersistMarkersByTableInner::new());
        }

        write_access
            .get_mut(table_name)
            .unwrap()
            .data_to_persist
            .mark_table_to_persist(sync_moment);
    }

    pub async fn persist_table_attrs(&self, table_name: &str) {
        let mut write_access = self.persist_by_table.lock().await;

        if !write_access.contains_key(table_name) {
            write_access.insert(table_name.to_string(), PersistMarkersByTableInner::new());
        }

        write_access
            .get_mut(table_name)
            .unwrap()
            .data_to_persist
            .mark_persist_attrs();
    }

    pub async fn get_job_to_persist(
        &self,
        table_name: &str,
        now: DateTimeAsMicroseconds,
        is_shutting_down: bool,
    ) -> Option<PersistResult> {
        let mut write_access = self.persist_by_table.lock().await;

        if !write_access.contains_key(table_name) {
            return None;
        }

        write_access
            .get_mut(table_name)
            .unwrap()
            .data_to_persist
            .get_what_to_persist(now, is_shutting_down)
    }

    pub async fn set_persisted(&self, table_name: &str, duration: Duration) {
        let mut write_access = self.persist_by_table.lock().await;

        if !write_access.contains_key(table_name) {
            write_access.insert(table_name.to_string(), PersistMarkersByTableInner::new());
        }

        let table = write_access.get_mut(table_name).unwrap();

        table.add_persist_duration(duration);
    }

    pub async fn get_persist_metrics(&self, table_name: &str) -> PersistMetrics {
        let read_access = self.persist_by_table.lock().await;

        match read_access.get(table_name) {
            Some(result) => PersistMetrics {
                last_persist_time: result.last_persist_time.clone(),
                next_persist_time: result.data_to_persist.get_next_persist_time(),
                persist_amount: result.data_to_persist.get_persist_amount(),
                last_persist_duration: result.persist_duration.clone(),
            },
            None => PersistMetrics {
                last_persist_time: None,
                next_persist_time: None,
                persist_amount: 0,
                last_persist_duration: vec![],
            },
        }
    }
}
