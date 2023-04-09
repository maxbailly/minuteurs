#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

//! A very lightweight crate to give users control as fine grained as possible over threads' execution over time at a minimal cost.
//!
//! # Timeouts
//!
//! A [`Timeout`] allow users to block a thread's execution until a certain amount of time passed since the creation of the timeout unless
//! the timeout already expired.
//!
//! It comes in two flavors:
//! * [`Timeout::once()`] returns a [`Timeout`] that can be triggered only once meaning that once such a timeout expires, it can never block
//!   anymore.
//! * [`Timeout::repeat()`] returns a [`Timeout`] that can be triggered multiple times with the timeout duration. In this case, if too much
//!   time have passed between two [`Timeout::wait()`] calls, it will try to catch up.
//!
//! ## Examples
//!
//! ### Basic example
//!
//! ```
//! use std::time::{Duration, Instant};
//! # use minuteurs::Timeout;
//!
//! // Create a new timeout of 1 second.
//! let mut timeout = Timeout::once(Duration::from_secs(1));
//! let mut now = Instant::now();
//!
//! // This sleep represents some heavy computation.
//! std::thread::sleep(Duration::from_millis(750));
//!
//! // Blocks the thread if less than 1 second have passed since the timemout's creation.
//! timeout.wait();
//!
//! // Until this point, at least 1 second have passed no matter what happened
//! // between the creation and the wait.
//! let elapsed = now.elapsed();
//! assert!(elapsed > Duration::from_secs(1));
//! println!("elapsed: {elapsed:?}");
//! ```
//!
//! Possible output:
//! ```text
//! elapsed: 1.00010838s
//! ```
//!
//! ### Using a timeout to synchronize multiple threads
//!
//! ```
//! use std::time::{Duration, Instant};
//! # use minuteurs::Timeout;
//!
//! // Create a repeatable timeout of 1 second.
//! let mut timeout = Timeout::repeat(Duration::from_secs(1));
//! let now = Instant::now();
//!
//! // Spawn two threads with the same timeout.
//! // They should prints approximatively every 1s.
//! let thread1 = std::thread::spawn(move || {
//!     for _ in 0..5 {
//!         timeout.wait();
//!         let elapsed = now.elapsed();
//!         println!("thread1 ticked at {elapsed:?}",)
//!     }
//! });
//! let thread2 = std::thread::spawn(move || {
//!     for _ in 0..5 {
//!         timeout.wait();
//!         let elapsed = now.elapsed();
//!         println!("thread2 ticked at {elapsed:?}",)
//!     }
//! });
//!
//! // Obligatory clean up.
//! let _ = thread1.join();
//! let _ = thread2.join();
//! ```
//!
//! Possible output:
//! ```text
//! thread2 ticked at 1.000112249s
//! thread1 ticked at 1.000112289s
//! thread2 ticked at 2.000107337s
//! thread1 ticked at 2.0001086s
//! thread2 ticked at 3.000101815s
//! thread1 ticked at 3.000641802s
//! thread1 ticked at 4.000100891s
//! thread2 ticked at 4.000100911s
//! thread2 ticked at 5.000106159s
//! thread1 ticked at 5.000112471s
//! ```
//!
//! # Timer
//!
//! A [`Timer`] differs from a repeatable [`Timeout`] in that a timer is specifically build to synchronize multiple threads on periodic
//! events and are more precise and better optimized.
//!
//! Usually, the timer runs in a loop in its own thread, while the [`Watcher`]s are passed in another threads.
//! The timer ticks periodically and notifies one or more watchers.
//!
//! ## Example
//!
//! ```
//! use std::sync::atomic::{AtomicBool, Ordering};
//! use std::sync::Arc;
//! use std::time::{Duration, Instant};
//! # use minuteurs::Timer;
//!
//! // Create a timer that ticks every seconds and get a watcher from it.
//! let mut timer = Timer::new(Duration::from_secs(1));
//! let mut watcher1 = timer.watcher();
//!
//! // Watchers are clonable and cloned ones are associated to the orignal watcher's timer.
//! // The timer will then notify the watcher2 as well.
//! let mut watcher2 = watcher1.clone();
//!
//! let now = Instant::now();
//! let stop = Arc::new(AtomicBool::default());
//!
//! // Spawn two threads.
//! // They should prints approximatively every 1s.
//! let stop_clone = Arc::clone(&stop);
//! let thread1 = std::thread::spawn(move || {
//!     while !stop_clone.load(Ordering::SeqCst) {
//!         if watcher1.has_ticked() {
//!             let elapsed = now.elapsed();
//!             println!("thread1 ticked at {elapsed:?}",)
//!         }
//!     }
//! });
//!
//! let stop_clone = Arc::clone(&stop);
//! let thread2 = std::thread::spawn(move || {
//!     while !stop_clone.load(Ordering::SeqCst) {
//!         if watcher2.has_ticked() {
//!             let elapsed = now.elapsed();
//!             println!("thread2 ticked at {elapsed:?}",)
//!         }
//!     }
//! });
//!
//! for _ in 0..5 {
//!     timer.tick();
//! }
//!
//! stop.store(true, Ordering::SeqCst);
//!
//! // Obligatory clean up.
//! let _ = thread1.join();
//! let _ = thread2.join();
//! ```
//!
//! Possible output:
//! ```text
//! thread1 ticked at 1.00087579s
//! thread2 ticked at 1.000878295s
//! thread1 ticked at 2.000870603s
//! thread2 ticked at 2.000873087s
//! thread2 ticked at 3.000875413s
//! thread1 ticked at 3.000876254s
//! thread2 ticked at 4.000874293s
//! thread1 ticked at 4.000875034s
//! thread2 ticked at 5.000874695s
//! thread1 ticked at 5.000875316s
//! ```

mod timeout;
mod timer;

pub use timeout::*;
pub use timer::*;
