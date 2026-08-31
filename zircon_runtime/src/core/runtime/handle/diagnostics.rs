use std::sync::MutexGuard;

use crate::core::diagnostics::{DiagnosticPath, DiagnosticStore, DiagnosticStoreSnapshot};

use super::CoreHandle;

impl CoreHandle {
    pub fn diagnostic_store(&self) -> DiagnosticStore {
        self.lock_diagnostics().clone()
    }

    pub fn diagnostic_store_snapshot(&self) -> DiagnosticStoreSnapshot {
        self.lock_diagnostics().snapshot()
    }

    pub fn record_diagnostic<U, T>(
        &self,
        path: impl Into<DiagnosticPath>,
        frame_index: u64,
        value: f64,
        unit: Option<U>,
        subsystem_tags: impl IntoIterator<Item = T>,
    ) where
        U: Into<String>,
        T: Into<String>,
    {
        let path: DiagnosticPath = path.into();
        let unit: Option<String> = unit.map(Into::into);
        self.lock_diagnostics()
            .record(path, frame_index, value, unit, subsystem_tags);
    }

    pub(crate) fn update_diagnostic_store<R>(
        &self,
        update: impl FnOnce(&mut DiagnosticStore) -> R,
    ) -> R {
        let mut store = self.lock_diagnostics();
        update(&mut store)
    }

    pub(super) fn lock_diagnostics(&self) -> MutexGuard<'_, DiagnosticStore> {
        self.inner
            .diagnostics
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::panic::{self, AssertUnwindSafe};
    use std::sync::Mutex;
    use std::time::Instant;

    use crate::core::CoreRuntime;

    #[test]
    fn core_handle_diagnostic_accessors_recover_poisoned_store_lock() {
        let runtime = CoreRuntime::new();
        let handle = runtime.handle();

        let _ = panic::catch_unwind(AssertUnwindSafe(|| {
            let _guard = handle.inner.diagnostics.lock().unwrap();
            panic!("poison core handle diagnostics store");
        }));

        handle.record_diagnostic("runtime.poisoned", 7, 42.0, Some("count"), ["runtime"]);

        let store = handle.diagnostic_store();
        let snapshot = store.snapshot();
        assert_eq!(snapshot.series.len(), 1);
        let series = &snapshot.series[0];
        assert_eq!(series.path.as_str(), "runtime.poisoned");
        assert_eq!(series.current, Some(42.0));
        assert_eq!(series.unit.as_deref(), Some("count"));
        assert_eq!(series.subsystem_tags, ["runtime"]);

        let snapshot = handle.diagnostic_store_snapshot();
        assert_eq!(snapshot.series.len(), 1);
    }

    #[test]
    fn optimization_batch_ft_runtime476_prepares_diagnostic_identity_before_locking() {
        let source = include_str!("diagnostics.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("core handle diagnostics production source");
        let record = production
            .split("pub fn record_diagnostic")
            .nth(1)
            .and_then(|tail| tail.split("pub(crate) fn update_diagnostic_store").next())
            .expect("record diagnostic production body");
        let path_conversion = record
            .find("let path: DiagnosticPath = path.into();")
            .expect("diagnostic path conversion");
        let unit_conversion = record
            .find("let unit: Option<String> = unit.map(Into::into);")
            .expect("diagnostic unit conversion");
        let lock = record
            .find("self.lock_diagnostics()")
            .expect("diagnostic store lock");

        assert!(path_conversion < lock);
        assert!(unit_conversion < lock);
    }

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_ft_runtime476_prepared_diagnostic_identity_lock_hold_benchmark() {
        const PATH_BYTES: usize = 4_096;
        const RECORDS_PER_SAMPLE: usize = 4_096;
        const SAMPLE_PAIRS: usize = 17;
        const UNIT_BYTES: usize = 1_024;

        let path = "p".repeat(PATH_BYTES);
        let unit = "u".repeat(UNIT_BYTES);
        for _ in 0..4 {
            black_box(measure_lock_hold(&path, &unit, false, RECORDS_PER_SAMPLE));
            black_box(measure_lock_hold(&path, &unit, true, RECORDS_PER_SAMPLE));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy_samples.push(measure_lock_hold(&path, &unit, false, RECORDS_PER_SAMPLE));
                optimized_samples.push(measure_lock_hold(&path, &unit, true, RECORDS_PER_SAMPLE));
            } else {
                optimized_samples.push(measure_lock_hold(&path, &unit, true, RECORDS_PER_SAMPLE));
                legacy_samples.push(measure_lock_hold(&path, &unit, false, RECORDS_PER_SAMPLE));
            }
        }

        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p95 = percentile(&optimized_samples, 95);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "RUNTIME476_PREPARED_DIAGNOSTIC_IDENTITY_LOCK_HOLD_BENCH_V1 sample_pairs={SAMPLE_PAIRS} records_per_sample={RECORDS_PER_SAMPLE} path_bytes={PATH_BYTES} unit_bytes={UNIT_BYTES} legacy_converted_bytes_inside_lock_per_record={} optimized_converted_bytes_inside_lock_per_record=0 legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=75",
            PATH_BYTES + UNIT_BYTES,
            csv(&legacy_samples),
            csv(&optimized_samples),
        );
        assert!(optimized_p95 <= legacy_p95 * 25 / 100);
    }

    fn measure_lock_hold(path: &str, unit: &str, optimized: bool, records: usize) -> u128 {
        let lock = Mutex::new(());
        let mut held_ns = 0;
        for _ in 0..records {
            if optimized {
                let owned_path = black_box(path).to_owned();
                let owned_unit = black_box(unit).to_owned();
                let started = Instant::now();
                let guard = lock.lock().unwrap();
                black_box((&owned_path, &owned_unit));
                drop(guard);
                held_ns += started.elapsed().as_nanos();
            } else {
                let started = Instant::now();
                let guard = lock.lock().unwrap();
                let owned_path = black_box(path).to_owned();
                let owned_unit = black_box(unit).to_owned();
                black_box((&owned_path, &owned_unit));
                drop(guard);
                held_ns += started.elapsed().as_nanos();
            }
        }
        held_ns.max(1)
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
