//! The [`Timeout`] implementation.

use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::time::{Duration, Instant};

/* ---------- */

/// A timeout that can either be triggered once or multiple times.
#[derive(Debug, Clone, Copy)]
pub struct Timeout {
    /// The kind of timeout.
    kind: TimeoutKind,
}

impl Timeout {
    /// Returns a new [`Timeout`] that will be triggered only once.
    #[inline]
    pub fn once(dur: Duration) -> Self {
        Self { kind: TimeoutKind::once(dur) }
    }

    /// Returns a new [`Timeout`] timeout that can be periodically triggered.
    #[inline]
    pub fn repeat(dur: Duration) -> Self {
        Self { kind: TimeoutKind::repeat(dur) }
    }

    /// Returns whether or not the [`Timeout`] expired.
    #[inline]
    pub fn expired(&mut self) -> bool {
        match &mut self.kind {
            TimeoutKind::Once(timeout) => timeout.expired(),
            TimeoutKind::Repeat(timeout) => timeout.expired(),
        }
    }

    /// Returns the remaining duration before expiration.
    #[inline]
    pub fn remaining_duration(&mut self) -> Duration {
        match &mut self.kind {
            TimeoutKind::Once(timeout) => timeout.remaining_duration(),
            TimeoutKind::Repeat(timeout) => timeout.remaining_duration(),
        }
    }

    /// Block the thread until the [`Timeout`] expires.
    #[inline]
    pub fn wait(&mut self) {
        match &mut self.kind {
            TimeoutKind::Once(timeout) => timeout.wait(),
            TimeoutKind::Repeat(timeout) => timeout.wait(),
        }
    }
}

/* ---------- */

/// Defines the various kind of timeouts.
#[derive(Clone, Copy)]
enum TimeoutKind {
    /// The variant of the timeout that can be triggered only once.
    Once(TimeoutOnce),
    /// The variant of the timeout that can be triggered repeatedly.
    Repeat(TimeoutRepeat),
}

impl TimeoutKind {
    /// Returns a timeout that can be triggered only once.
    #[inline]
    fn once(dur: Duration) -> Self {
        Self::Once(TimeoutOnce::new(dur))
    }

    /// Returns a timeout that can be triggered repeatedly.
    #[inline]
    fn repeat(dur: Duration) -> Self {
        Self::Repeat(TimeoutRepeat::new(dur))
    }
}

impl Debug for TimeoutKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Once(inner) => write!(f, "{inner:?}"),
            Self::Repeat(inner) => write!(f, "{inner:?}"),
        }
    }
}

/* ---------- */

/// A timeout that is triggered only once.
#[derive(Debug, Clone, Copy)]
struct TimeoutOnce {
    /// The time when the timeout is triggered.
    delivery_time: Instant,
}

impl TimeoutOnce {
    /// Returns a new [`TimeoutOnce`] triggered after `dur` time.
    #[inline]
    fn new(dur: Duration) -> Self {
        let delivery_time = checked_delivery_time(Instant::now(), dur);
        Self { delivery_time }
    }

    /// Returns whether or not the timeout expired.
    ///
    /// Once the timeout expires, it always returns true.
    #[inline]
    fn expired(&self) -> bool {
        self.remaining_duration() == Duration::ZERO
    }

    /// Returns the time before the next trigger.
    ///
    /// Once the timeout expires, it always returns [`Duration::ZERO`].
    #[inline]
    fn remaining_duration(&self) -> Duration {
        self.delivery_time - Instant::now()
    }

    /// Waits until the timeout expires.
    #[inline]
    fn wait(&self) {
        std::thread::sleep(self.remaining_duration())
    }
}

/* ---------- */

/// A timeout that can be periodically triggered.
#[derive(Debug, Clone, Copy)]
struct TimeoutRepeat {
    /// The period bewteen each trigger.
    dur: Duration,
    /// The time when the timeout is triggered.
    delivery_time: Instant,
}

impl TimeoutRepeat {
    /// Returns a new [`TimeoutRepeat`] triggered after `dur` time.
    #[inline]
    fn new(dur: Duration) -> Self {
        let delivery_time = checked_delivery_time(Instant::now(), dur);
        Self { dur, delivery_time }
    }

    /// Returns whether or not the timeout expired.
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

    /// Waits until the timeout expires.
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
        let mut timeout = Timeout::once(Duration::from_millis(100));

        assert!(!timeout.expired());

        std::thread::sleep(Duration::from_millis(110));
        assert!(timeout.expired());
        assert!(timeout.expired());
    }

    #[test]
    fn once_remains() {
        let mut timeout = Timeout::once(Duration::from_millis(100));
        assert!(timeout.remaining_duration() > Duration::ZERO);
        assert!(timeout.remaining_duration() < Duration::from_millis(100));

        std::thread::sleep(Duration::from_millis(50));
        assert!(timeout.remaining_duration() > Duration::ZERO);
        assert!(timeout.remaining_duration() < Duration::from_millis(50));

        std::thread::sleep(Duration::from_millis(51));
        assert!(timeout.remaining_duration() == Duration::ZERO);
        assert!(timeout.remaining_duration() == Duration::ZERO);
    }

    #[test]
    fn once_wait() {
        let mut timeout = Timeout::once(Duration::from_millis(100));
        let now = Instant::now();
        timeout.wait();
        assert!(now.elapsed() >= Duration::from_millis(100));

        let mut timeout = Timeout::once(Duration::from_millis(100));
        let now = Instant::now();
        std::thread::sleep(Duration::from_millis(50));
        timeout.wait();
        let delay = now.elapsed();
        assert!(delay >= Duration::from_millis(100));
        assert!(delay < Duration::from_millis(110));

        let now = Instant::now();
        timeout.wait();
        let delay = now.elapsed();
        assert!(delay < Duration::from_millis(1));
    }

    #[test]
    fn repeat_expired() {
        let mut timeout = Timeout::repeat(Duration::from_millis(100));

        assert!(!timeout.expired());

        std::thread::sleep(Duration::from_millis(110));
        assert!(timeout.expired());
        assert!(!timeout.expired());
    }

    #[test]
    fn repeat_remains() {
        let mut timeout = Timeout::repeat(Duration::from_millis(100));
        assert!(timeout.remaining_duration() > Duration::ZERO);
        assert!(timeout.remaining_duration() < Duration::from_millis(100));

        std::thread::sleep(Duration::from_millis(50));
        assert!(timeout.remaining_duration() > Duration::ZERO);
        assert!(timeout.remaining_duration() < Duration::from_millis(50));

        std::thread::sleep(Duration::from_millis(51));
        assert!(timeout.remaining_duration() == Duration::ZERO);
        assert!(timeout.remaining_duration() < Duration::from_millis(100));
    }

    #[test]
    fn repeat_wait() {
        let mut timeout = Timeout::repeat(Duration::from_millis(100));
        let now = Instant::now();
        timeout.wait();
        assert!(now.elapsed() >= Duration::from_millis(100));

        let mut timeout = Timeout::repeat(Duration::from_millis(100));
        let now = Instant::now();
        std::thread::sleep(Duration::from_millis(50));
        timeout.wait();
        let delay = now.elapsed();
        assert!(delay >= Duration::from_millis(100));
        assert!(delay < Duration::from_millis(110));

        let now = Instant::now();
        timeout.wait();
        let delay = now.elapsed();
        assert!(delay >= Duration::from_millis(90), "delay = {:?}", delay);
    }
}
