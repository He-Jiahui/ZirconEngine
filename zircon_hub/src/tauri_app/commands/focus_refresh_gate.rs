use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

#[derive(Clone, Default)]
pub(super) struct FocusRefreshGate {
    pending: Arc<AtomicBool>,
}

impl FocusRefreshGate {
    pub(super) fn try_enter(&self) -> Option<FocusRefreshPermit> {
        if self.pending.load(Ordering::Acquire) {
            return None;
        }
        self.pending
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .ok()?;
        Some(FocusRefreshPermit {
            pending: Arc::clone(&self.pending),
        })
    }
}

pub(super) struct FocusRefreshPermit {
    pending: Arc<AtomicBool>,
}

impl Drop for FocusRefreshPermit {
    fn drop(&mut self) {
        self.pending.store(false, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use std::{
        hint::black_box,
        panic::catch_unwind,
        sync::atomic::{AtomicBool, Ordering},
        time::Instant,
    };

    use super::FocusRefreshGate;

    #[test]
    fn hub05_focus_refresh_gate_rejects_duplicates_until_drop() {
        let gate = FocusRefreshGate::default();
        let permit = gate.try_enter().expect("first focus refresh enters");

        assert!(gate.try_enter().is_none());
        drop(permit);
        assert!(gate.try_enter().is_some());
    }

    #[test]
    fn hub05_focus_refresh_gate_releases_after_worker_panic() {
        let gate = FocusRefreshGate::default();
        let worker_gate = gate.clone();

        let result = catch_unwind(move || {
            let _permit = worker_gate.try_enter().expect("worker enters");
            panic!("simulated focus refresh worker panic");
        });

        assert!(result.is_err());
        assert!(gate.try_enter().is_some());
    }

    #[test]
    #[ignore = "managed release performance contract"]
    fn hub05_focus_refresh_gate_release_benchmark_evidence() {
        const ATTEMPTS: usize = 2_000_000;
        const SAMPLE_PAIRS: usize = 21;
        const THRESHOLD_PERCENT: u128 = 30;

        let legacy_pending = AtomicBool::new(true);
        let optimized_gate = FocusRefreshGate::default();
        let _optimized_permit = optimized_gate.try_enter().expect("benchmark gate enters");

        for _ in 0..4 {
            black_box(measure_rejections(50_000, || {
                !legacy_pending.swap(true, Ordering::AcqRel)
            }));
            black_box(measure_rejections(50_000, || {
                optimized_gate.try_enter().is_some()
            }));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            let measure_legacy =
                || measure_rejections(ATTEMPTS, || !legacy_pending.swap(true, Ordering::AcqRel));
            let measure_optimized =
                || measure_rejections(ATTEMPTS, || optimized_gate.try_enter().is_some());
            if pair % 2 == 0 {
                legacy_samples.push(measure_legacy());
                optimized_samples.push(measure_optimized());
            } else {
                optimized_samples.push(measure_optimized());
                legacy_samples.push(measure_legacy());
            }
        }

        assert!(legacy_samples.iter().all(|sample| sample.accepted == 0));
        assert!(optimized_samples.iter().all(|sample| sample.accepted == 0));
        let legacy_ns = elapsed_samples(&legacy_samples);
        let optimized_ns = elapsed_samples(&optimized_samples);
        let legacy_p50_ns = percentile(&legacy_ns, 50);
        let legacy_p95_ns = percentile(&legacy_ns, 95);
        let optimized_p50_ns = percentile(&optimized_ns, 50);
        let optimized_p95_ns = percentile(&optimized_ns, 95);
        let p50_reduction_percent = reduction_percent(legacy_p50_ns, optimized_p50_ns);
        let p95_reduction_percent = reduction_percent(legacy_p95_ns, optimized_p95_ns);

        println!(
            "PERF_RESULT hub05_focus_refresh_gate attempts=2000000 sample_pairs=21 \
             threshold_percent=30 legacy_writes_per_attempt=1 optimized_writes_per_attempt=0 \
             legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
             p50_reduction_percent={p50_reduction_percent} \
             legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
             p95_reduction_percent={p95_reduction_percent} \
             legacy_raw_ns={} optimized_raw_ns={}",
            raw_samples(&legacy_ns),
            raw_samples(&optimized_ns),
        );

        assert!(
            p50_reduction_percent >= THRESHOLD_PERCENT,
            "read-fast rejection must improve P50 by at least {THRESHOLD_PERCENT}%: \
             legacy={legacy_p50_ns}ns optimized={optimized_p50_ns}ns"
        );
        assert!(
            p95_reduction_percent >= THRESHOLD_PERCENT,
            "read-fast rejection must improve P95 by at least {THRESHOLD_PERCENT}%: \
             legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }

    #[derive(Clone, Copy)]
    struct Measurement {
        elapsed_ns: u128,
        accepted: usize,
    }

    fn measure_rejections(attempts: usize, mut attempt: impl FnMut() -> bool) -> Measurement {
        let started = Instant::now();
        let mut accepted = 0usize;
        for _ in 0..attempts {
            accepted += black_box(attempt()) as usize;
        }
        Measurement {
            elapsed_ns: started.elapsed().as_nanos().max(1),
            accepted,
        }
    }

    fn elapsed_samples(measurements: &[Measurement]) -> Vec<u128> {
        measurements
            .iter()
            .map(|measurement| measurement.elapsed_ns)
            .collect()
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn reduction_percent(legacy: u128, optimized: u128) -> u128 {
        legacy.saturating_sub(optimized).saturating_mul(100) / legacy.max(1)
    }

    fn raw_samples(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
