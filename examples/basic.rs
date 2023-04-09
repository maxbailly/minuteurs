use std::time::{Duration, Instant};

use minuteurs::Timeout;

fn main() {
    let mut timeout = Timeout::once(Duration::from_secs(1));
    let now = Instant::now();

    std::thread::sleep(Duration::from_millis(750));

    timeout.wait();

    let elapsed = now.elapsed();
    assert!(elapsed > Duration::from_secs(1));
    println!("elapsed: {elapsed:?}");
}
