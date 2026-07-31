use std::time::{Duration, Instant};

pub(super) const LOAD_WINDOW: Duration = Duration::from_secs(1);
const PACING_BUCKETS: usize = 100;

pub(super) fn event_offset(sequence: usize, logs_per_second: usize) -> Duration {
    let bucket = sequence.saturating_mul(PACING_BUCKETS) / logs_per_second;
    let nanos = (bucket as u128).saturating_mul(LOAD_WINDOW.as_nanos()) / PACING_BUCKETS as u128;
    Duration::from_nanos(u64::try_from(nanos).unwrap_or(u64::MAX))
}

pub(super) fn wait_until(deadline: Instant) {
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return;
        }
        if remaining > Duration::from_millis(1) {
            std::thread::sleep(remaining - Duration::from_micros(200));
        } else if remaining > Duration::from_micros(50) {
            std::thread::yield_now();
        } else {
            std::hint::spin_loop();
        }
    }
}
