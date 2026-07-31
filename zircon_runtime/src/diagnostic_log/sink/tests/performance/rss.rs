use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

#[cfg(windows)]
mod windows;

#[cfg(windows)]
use windows::working_set_bytes;

#[derive(Clone, Copy)]
pub(super) struct RssSnapshot {
    pub(super) baseline: Option<u64>,
    pub(super) peak: Option<u64>,
    pub(super) after: Option<u64>,
    pub(super) sample_count: u64,
}

impl RssSnapshot {
    pub(super) fn peak_growth(self) -> Option<u64> {
        Some(self.peak?.saturating_sub(self.baseline?))
    }
}

pub(super) struct RssSampler {
    baseline: Option<u64>,
    peak: Arc<AtomicU64>,
    sample_count: Arc<AtomicU64>,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl RssSampler {
    pub(super) fn start() -> Self {
        let baseline = working_set_bytes();
        let peak = Arc::new(AtomicU64::new(0));
        let sample_count = Arc::new(AtomicU64::new(0));
        let stop = Arc::new(AtomicBool::new(false));
        let worker_peak = Arc::clone(&peak);
        let worker_sample_count = Arc::clone(&sample_count);
        let worker_stop = Arc::clone(&stop);
        let worker = std::thread::Builder::new()
            .name("perf-mvp-434-rss".to_string())
            .spawn(move || {
                while !worker_stop.load(Ordering::Acquire) {
                    if let Some(sample) = working_set_bytes() {
                        worker_peak.fetch_max(sample, Ordering::Relaxed);
                        worker_sample_count.fetch_add(1, Ordering::Relaxed);
                    }
                    std::thread::sleep(Duration::from_millis(10));
                }
            })
            .expect("RSS sampler thread");
        Self {
            baseline,
            peak,
            sample_count,
            stop,
            worker: Some(worker),
        }
    }

    pub(super) fn finish(mut self) -> RssSnapshot {
        self.stop_and_join();
        let peak =
            (self.peak.load(Ordering::Relaxed) != 0).then_some(self.peak.load(Ordering::Relaxed));
        RssSnapshot {
            baseline: self.baseline,
            peak,
            after: working_set_bytes(),
            sample_count: self.sample_count.load(Ordering::Relaxed),
        }
    }

    fn stop_and_join(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            worker.join().unwrap();
        }
    }
}

impl Drop for RssSampler {
    fn drop(&mut self) {
        self.stop_and_join();
    }
}

#[cfg(not(windows))]
fn working_set_bytes() -> Option<u64> {
    None
}
