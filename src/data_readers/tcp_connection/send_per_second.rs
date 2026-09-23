use std::collections::VecDeque;

use parking_lot::Mutex;

const MAX_DATA_LEN: usize = 120;

/// Bytes sent to a reader per second, for the last `MAX_DATA_LEN` seconds.
pub struct SendPerSecond {
    data: Mutex<VecDeque<usize>>,
}

impl SendPerSecond {
    pub fn new() -> Self {
        Self {
            data: Mutex::new(VecDeque::with_capacity(MAX_DATA_LEN + 1)),
        }
    }

    pub fn add(&self, value: usize) {
        let mut write_access = self.data.lock();
        write_access.push_back(value);

        while write_access.len() > MAX_DATA_LEN {
            write_access.pop_front();
        }
    }

    pub fn get_snapshot(&self) -> Vec<usize> {
        self.data.lock().iter().copied().collect()
    }
}
