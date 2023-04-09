use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use timings::Timer;

/* ---------- */

fn main() {
    // Create a timer that ticks every seconds and get a watcher from it.
    let mut timer = Timer::new(Duration::from_secs(1));
    let mut watcher1 = timer.watcher();

    // Watchers are clonable and cloned ones are associated to the orignal watcher's timer.
    // The timer will then notify the watcher2 as well.
    let mut watcher2 = watcher1.clone();

    let now = Instant::now();
    let stop = Arc::new(AtomicBool::default());

    // Spawn two threads.
    // They should prints approximatively every 1s.
    let stop_clone = Arc::clone(&stop);
    let thread1 = std::thread::spawn(move || {
        while !stop_clone.load(Ordering::SeqCst) {
            if watcher1.has_ticked() {
                let elapsed = now.elapsed();
                println!("thread1 ticked at {elapsed:?}",)
            }
        }
    });

    let stop_clone = Arc::clone(&stop);
    let thread2 = std::thread::spawn(move || {
        while !stop_clone.load(Ordering::SeqCst) {
            if watcher2.has_ticked() {
                let elapsed = now.elapsed();
                println!("thread2 ticked at {elapsed:?}",)
            }
        }
    });

    for _ in 0..5 {
        timer.tick();
    }

    stop.store(true, Ordering::SeqCst);

    // Obligatory clean up.
    let _ = thread1.join();
    let _ = thread2.join();
}
