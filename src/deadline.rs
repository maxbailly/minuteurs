//! The [`Deadline`] implementation.

use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::time::{Duration, Instant};

/* ---------- */

/// A deadline that can either be triggered once or multiple times.
#[derive(Debug, Clone, Copy)]
pub struct Deadline {
    /// The kind of deadline.
    kind: DeadlineKind,
}

impl Deadline {
    /// Returns a new [`Deadline`] that will be triggered only once.
    #[inline]
    pub fn once(dur: Duration) -> Self {
        Self {
            kind: DeadlineKind::once(dur),
        }
    }

    /// Returns a new [`Deadline`] that can be periodically triggered.
    #[inline]
    pub fn repeat(dur: Duration) -> Self {
        Self {
            kind: DeadlineKind::repeat(dur),
        }
    }

    /// Returns whether or not the [`Deadline`] expired.
    #[inline]
    pub fn expired(&mut self) -> bool {
        match &mut self.kind {
            DeadlineKind::Once(deadline) => deadline.expired(),
            DeadlineKind::Repeat(deadline) => deadline.expired(),
        }
    }

    /// Returns the remaining duration before expiration.
    #[inline]
    pub fn remaining_duration(&mut self) -> Duration {
        match &mut self.kind {
            DeadlineKind::Once(deadline) => deadline.remaining_duration(),
            DeadlineKind::Repeat(deadline) => deadline.remaining_duration(),
        }
    }

    /// Block the thread until the [`Deadline`] expires.
    #[inline]
    pub fn wait(&mut self) {
        match &mut self.kind {
            DeadlineKind::Once(deadline) => deadline.wait(),
            DeadlineKind::Repeat(deadline) => deadline.wait(),
        }
    }
}

/* ---------- */

/// Defines the various kind of deadlines.
#[derive(Clone, Copy)]
enum DeadlineKind {
    /// The variant of the deadline that can be triggered only once.
    Once(DeadlineOnce),
    /// The variant of the deadline that can be triggered repeatedly.
    Repeat(DeadlineRepeat),
}

impl DeadlineKind {
    /// Returns a deadline that can be triggered only once.
    #[inline]
    fn once(dur: Duration) -> Self {
        Self::Once(DeadlineOnce::new(dur))
    }

    /// Returns a deadline that can be triggered repeatedly.
    #[inline]
    fn repeat(dur: Duration) -> Self {
        Self::Repeat(DeadlineRepeat::new(dur))
    }
}

impl Debug for DeadlineKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Once(inner) => write!(f, "{inner:?}"),
            Self::Repeat(inner) => write!(f, "{inner:?}"),
        }
    }
}

/* ---------- */

/// A deadline that is triggered only once.
#[derive(Debug, Clone, Copy)]
struct DeadlineOnce {
    /// The time when the deadline is triggered.
    delivery_time: Instant,
}

impl DeadlineOnce {
    /// Returns a new [`DeadlineOnce`] triggered after `dur` time.
    #[inline]
    fn new(dur: Duration) -> Self {
        let delivery_time = checked_delivery_time(Instant::now(), dur);
        Self { delivery_time }
    }

    /// Returns whether or not the deadline expired.
    ///
    /// Once the deadline expires, it always returns true.
    #[inline]
    fn expired(&self) -> bool {
        self.remaining_duration() == Duration::ZERO
    }

    /// Returns the time before the next trigger.
    ///
    /// Once the deadline expires, it always returns [`Duration::ZERO`].
    #[inline]
    fn remaining_duration(&self) -> Duration {
        self.delivery_time - Instant::now()
    }

    /// Waits until the deadline expires.
    #[inline]
    fn wait(&self) {
        std::thread::sleep(self.remaining_duration())
    }
}

/* ---------- */

/// A deadline that can be periodically triggered.
#[derive(Debug, Clone, Copy)]
struct DeadlineRepeat {
    /// The period bewteen each trigger.
    dur: Duration,
    /// The time when the deadline is triggered.
    delivery_time: Instant,
}

impl DeadlineRepeat {
    /// Returns a new [`DeadlineRepeat`] triggered after `dur` time.
    #[inline]
    fn new(dur: Duration) -> Self {
        let delivery_time = checked_delivery_time(Instant::now(), dur);
        Self { dur, delivery_time }
    }

    /// Returns whether or not the deadline expired.
    #[inline]
    fn expired(&mut self) -> bool {
        self.remaining_duration() == Duration::ZERO
    }

    /// Returns the time before the next trigger.
    #[inline]
    fn remaining_duration(&mut self) -> Duration {
        let ret = self.delivery_time - Instant::now();

        if ret == Duration::ZERO {
            self.delivery_time += self.dur;
        }

        ret
    }

    /// Waits until the deadline expires.
    #[inline]
    fn wait(&mut self) {
        std::thread::sleep(self.remaining_duration());
        self.delivery_time += self.dur;
    }
}

/* ---------- */

/// Returns the next delivery time.
///
/// If the given dur is too large, we set the next delivery time to
/// the next decade.
#[inline]
fn checked_delivery_time(instant: Instant, dur: Duration) -> Instant {
    /// Represents a decade.
    const TEN_YEARS: Duration = Duration::from_secs(86400 * 365 * 10);

    instant.checked_add(dur).unwrap_or(instant + TEN_YEARS)
}

/* ---------- */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delivery_time() {
        let now = Instant::now();

        let ret = checked_delivery_time(now, Duration::from_secs(1));
        assert_eq!(ret, now + Duration::from_secs(1));

        let ret = checked_delivery_time(now, Duration::MAX);
        assert_eq!(ret, now + Duration::from_secs(86400 * 365 * 10));
    }

    #[test]
    fn once_expired() {
        let mut deadline = Deadline::once(Duration::from_millis(100));

        assert!(!deadline.expired());

        std::thread::sleep(Duration::from_millis(110));
        assert!(deadline.expired());
        assert!(deadline.expired());
    }

    #[test]
    fn once_remains() {
        let mut deadline = Deadline::once(Duration::from_millis(100));
        assert!(deadline.remaining_duration() > Duration::ZERO);
        assert!(deadline.remaining_duration() < Duration::from_millis(100));

        std::thread::sleep(Duration::from_millis(50));
        assert!(deadline.remaining_duration() > Duration::ZERO);
        assert!(deadline.remaining_duration() < Duration::from_millis(50));

        std::thread::sleep(Duration::from_millis(51));
        assert!(deadline.remaining_duration() == Duration::ZERO);
        assert!(deadline.remaining_duration() == Duration::ZERO);
    }

    #[test]
    fn once_wait() {
        let mut deadline = Deadline::once(Duration::from_millis(100));
        let now = Instant::now();
        deadline.wait();
        assert!(now.elapsed() >= Duration::from_millis(100));

        let mut deadline = Deadline::once(Duration::from_millis(100));
        let now = Instant::now();
        std::thread::sleep(Duration::from_millis(50));
        deadline.wait();
        let delay = now.elapsed();
        assert!(delay >= Duration::from_millis(100));
        assert!(delay < Duration::from_millis(110));

        let now = Instant::now();
        deadline.wait();
        let delay = now.elapsed();
        assert!(delay < Duration::from_millis(1));
    }

    #[test]
    fn repeat_expired() {
        let mut deadline = Deadline::repeat(Duration::from_millis(100));

        assert!(!deadline.expired());

        std::thread::sleep(Duration::from_millis(110));
        assert!(deadline.expired());
        assert!(!deadline.expired());
    }

    #[test]
    fn repeat_remains() {
        let mut deadline = Deadline::repeat(Duration::from_millis(100));
        assert!(deadline.remaining_duration() > Duration::ZERO);
        assert!(deadline.remaining_duration() < Duration::from_millis(100));

        std::thread::sleep(Duration::from_millis(50));
        assert!(deadline.remaining_duration() > Duration::ZERO);
        assert!(deadline.remaining_duration() < Duration::from_millis(50));

        std::thread::sleep(Duration::from_millis(51));
        assert!(deadline.remaining_duration() == Duration::ZERO);
        assert!(deadline.remaining_duration() < Duration::from_millis(100));
    }

    #[test]
    fn repeat_wait() {
        let mut deadline = Deadline::repeat(Duration::from_millis(100));
        let now = Instant::now();
        deadline.wait();
        assert!(now.elapsed() >= Duration::from_millis(100));

        let mut deadline = Deadline::repeat(Duration::from_millis(100));
        let now = Instant::now();
        std::thread::sleep(Duration::from_millis(50));
        deadline.wait();
        let delay = now.elapsed();
        assert!(delay >= Duration::from_millis(100));
        assert!(delay < Duration::from_millis(110));

        let now = Instant::now();
        deadline.wait();
        let delay = now.elapsed();
        assert!(delay >= Duration::from_millis(90), "delay = {:?}", delay);
    }
}
