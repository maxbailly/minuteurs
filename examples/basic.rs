use std::time::{Duration, Instant};

use minuteurs::Deadline;

fn main() {
    let mut deadline = Deadline::once(Duration::from_secs(1));
    let now = Instant::now();

    std::thread::sleep(Duration::from_millis(750));

    deadline.wait();

    let elapsed = now.elapsed();
    assert!(elapsed > Duration::from_secs(1));
    println!("elapsed: {elapsed:?}");
}
