use std::sync::atomic::{AtomicUsize, Ordering};

/// A value accumulated during a second and reported for the second before - ticked once a
/// second by `MetricsUpdater`.
#[derive(Default)]
pub struct PerSecondCounter {
    accumulated: AtomicUsize,
    last_second: AtomicUsize,
}

impl PerSecondCounter {
    pub fn add(&self, value: usize) {
        self.accumulated.fetch_add(value, Ordering::Relaxed);
    }

    pub fn one_second_tick(&self) {
        let value = self.accumulated.swap(0, Ordering::Relaxed);
        self.last_second.store(value, Ordering::Relaxed);
    }

    pub fn get(&self) -> usize {
        self.last_second.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::PerSecondCounter;

    #[test]
    fn test_reports_the_previous_second() {
        let counter = PerSecondCounter::default();

        counter.add(5);
        counter.add(3);
        assert_eq!(0, counter.get());

        counter.one_second_tick();
        assert_eq!(8, counter.get());

        counter.one_second_tick();
        assert_eq!(0, counter.get());
    }
}
