use std::time::{Duration, Instant};

use timings::Timeout;

/* ---------- */

fn main() {
    let mut timeout = Timeout::repeat(Duration::from_secs(1));
    let now = Instant::now();

    let thread1 = std::thread::spawn(move || {
        for _ in 0..10 {
            timeout.wait();
            let elapsed = now.elapsed();
            println!("thread1 ticked at {elapsed:?}",)
        }
    });

    let thread2 = std::thread::spawn(move || {
        for _ in 0..10 {
            timeout.wait();
            let elapsed = now.elapsed();
            println!("thread2 ticked at {elapsed:?}",)
        }
    });

    let _ = thread1.join();
    let _ = thread2.join();
}
