//! Types relative to the timer feature.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::Deadline;

/* ---------- */

/// A timer that ticks at a periodic time.
///
/// On missing ticks, the timer will burst until it catches up
/// with the defined delay.
#[derive(Debug)]
pub struct Timer {
    /// The inner state of the timer, toggle on each ticks.
    state: State,

    /// The deadline used to trigger the timer's ticks.
    deadline: Deadline,
}

impl Timer {
    /// Returns a new timer that ticks every `delay`.
    pub fn new(delay: Duration) -> Self {
        Self {
            state: State::new(),
            deadline: Deadline::repeat(delay),
        }
    }

    /// Returns a new watcher associated to `self`.
    pub fn watcher(&self) -> Watcher {
        Watcher::new(self.state.clone())
    }

    /// Blocks the current thread until the next tick and notify the associated watchers.
    pub fn tick(&mut self) {
        self.deadline.wait();
        self.state.toggle();
    }
}

/* ---------- */

/// A handle associated to a [`Timer`] that is notified when the timer ticks.
///
/// Watchers are safely clonable. A cloned watcher will be associated to the
/// [`Timer`] of the original one. This is equivalent to calling [`Timer::watcher()`] twice.
pub struct Watcher {
    /// The inner state of the associated [`Timer`].
    state: State,

    /// The prévious value of the state.
    prev_state: bool,
}

impl Watcher {
    /// Returns a new watcher associated to a [`Timer`].
    fn new(state: State) -> Self {
        let prev_state = state.value();
        Self { state, prev_state }
    }

    /// Returns whether or not the associated [`Timer`] has ticked.
    pub fn has_ticked(&mut self) -> bool {
        if self.state.value() != self.prev_state {
            self.prev_state = !self.prev_state;
            return true;
        }

        false
    }
}

impl Clone for Watcher {
    fn clone(&self) -> Self {
        // FIXME: What happens if the timer ticks between the the clone and the prev_state ?
        // We might miss 2 ticks here, is it a problem ?
        // We probably should get the prev_value *before* the clone itself.
        let state = self.state.clone();
        let prev_state = state.value();

        Self { state, prev_state }
    }
}

/* ---------- */

/// Inner state of the [`Timer`] and [`Watcher`] types.
#[derive(Debug, Default, Clone)]
struct State(Arc<AtomicBool>);

impl State {
    /// Returns a new state with a default value.
    #[inline]
    fn new() -> Self {
        Self::default()
    }

    /// Flip the state's value.
    #[inline]
    fn toggle(&self) {
        self.0.fetch_xor(true, Ordering::Release);
    }

    /// Returns the state's inner value.
    #[inline]
    fn value(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

#[cfg(test)]
impl PartialEq<bool> for State {
    #[inline]
    fn eq(&self, other: &bool) -> bool {
        self.value() == *other
    }
}

/* ---------- */

#[cfg(test)]
mod state {
    use super::*;

    #[test]
    fn new() {
        let new = State::new();
        assert_eq!(new, false);
    }

    #[test]
    fn toggle() {
        let new = State::new();
        assert_eq!(new, false);

        new.toggle();
        assert_eq!(new, true);
    }
}

#[cfg(test)]
#[allow(clippy::module_inception)]
mod timer {
    use std::time::Instant;

    use super::*;

    #[test]
    fn tick_delay() {
        let now = Instant::now();
        let mut timer = Timer::new(Duration::from_millis(100));

        for count in 0..5 {
            timer.tick();

            let elapsed = now.elapsed();
            assert!(
                now.elapsed() >= Duration::from_millis(100 * count),
                "elapsed = {elapsed:?}"
            )
        }
    }
}

#[cfg(test)]
mod watcher {
    use std::time::Instant;

    use super::*;

    #[test]
    fn new() {
        let mut timer = Timer::new(Duration::from_millis(100));
        let mut watcher = timer.watcher();

        assert!(
            !watcher.has_ticked(),
            "watcher shouldn't have been notified yet"
        );

        timer.tick();
        assert!(watcher.has_ticked(), "watcher should have been notified");
        assert!(
            !watcher.has_ticked(),
            "watcher shouldn't have been notified instantly"
        );
    }

    #[test]
    fn cloned() {
        let mut timer = Timer::new(Duration::from_millis(100));

        let mut watcher = timer.watcher();
        assert!(
            !watcher.has_ticked(),
            "watcher shouldn't have been notified yet"
        );

        let mut watcher_clone = watcher.clone();
        assert!(
            !watcher_clone.has_ticked(),
            "watcher clone shouldn't have been notified yet"
        );

        timer.tick();
        assert!(watcher.has_ticked(), "watcher should have been notified");
        assert!(
            watcher_clone.has_ticked(),
            "watcher clone should have been notified"
        );

        let mut watcher_clone = watcher.clone();
        assert!(
            !watcher_clone.has_ticked(),
            "2dn watcher clone shouldn't have been notified yet"
        );
    }

    #[test]
    fn thread_sync() {
        let stop = Arc::new(AtomicBool::default());
        let now = Instant::now();
        let mut timer = Timer::new(Duration::from_millis(100));
        let mut watcher = timer.watcher();

        let stop_clone = Arc::clone(&stop);
        let watcher_thread = std::thread::spawn(move || {
            let mut loops = 1;

            while !stop_clone.load(Ordering::Acquire) {
                if watcher.has_ticked() {
                    let elapsed = now.elapsed();
                    let expected = Duration::from_millis(100 * loops);

                    if now.elapsed() < expected {
                        return Some((elapsed, expected));
                    }

                    loops += 1;
                }
            }

            None
        });

        for _ in 0..5 {
            timer.tick();
        }

        stop.store(true, Ordering::Release);
        let test_result = watcher_thread.join().unwrap();
        assert_eq!(
            test_result, None,
            "watcher detected a tick before the expected time"
        );
    }
}
