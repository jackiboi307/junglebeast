use std::time::{Duration, SystemTime};

pub struct Interval {
    interval: u64,
    last: u64,
    delta: Duration,
}

impl Interval {
    pub fn new(duration: Duration) -> Self {
        Self {
            interval: duration.as_millis().try_into().unwrap(),
            last: 0,
            delta: Duration::ZERO,
        }
    }

    pub fn tick(&mut self) -> bool {
        let now = SystemTime::UNIX_EPOCH.elapsed().unwrap();
        let now: u64 = now.as_millis().try_into().unwrap();

        if self.last + self.interval < now {
            self.delta =
                if self.last != 0 {
                    Duration::from_millis(now - self.last)
                } else { Duration::ZERO };
            self.last = now;
            return true;
        }

        return false;
    }

    pub fn delta(&self) -> Duration {
        self.delta
    }
}
