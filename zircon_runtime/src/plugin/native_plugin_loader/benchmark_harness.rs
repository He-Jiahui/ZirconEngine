use std::fmt::Write as _;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier};
use std::thread::{self, Thread};
use std::time::Duration;

const SOURCE_MANIFEST_ENV: &str = "ZR_BENCHMARK_SOURCE_MANIFEST";
const CARGO_PROFILE_ENV: &str = "ZR_BENCHMARK_CARGO_PROFILE";
pub(super) const BENCHMARK_RECORD_SCHEMA: &str = "zircon.native.benchmark/2";

/// Two-phase start boundary that excludes worker startup from a timed benchmark interval.
pub(super) struct BenchmarkWorkerStartGate {
    ready: Arc<Barrier>,
    start: Arc<Barrier>,
    owner: Thread,
}

impl BenchmarkWorkerStartGate {
    pub(super) fn new(worker_count: usize) -> Self {
        assert!(worker_count > 0, "benchmark must have at least one worker");
        Self {
            ready: Arc::new(Barrier::new(worker_count + 1)),
            start: Arc::new(Barrier::new(worker_count + 1)),
            owner: thread::current(),
        }
    }

    pub(super) fn worker_start(&self) -> BenchmarkWorkerStart {
        BenchmarkWorkerStart {
            ready: Arc::clone(&self.ready),
            start: Arc::clone(&self.start),
        }
    }

    pub(super) fn wait_until_ready(&self) {
        self.assert_owner();
        self.ready.wait();
    }

    pub(super) fn start(&self) {
        self.assert_owner();
        self.start.wait();
    }

    fn assert_owner(&self) {
        assert_eq!(
            thread::current().id(),
            self.owner.id(),
            "benchmark start gate must be driven by its owner thread"
        );
    }
}

pub(super) struct BenchmarkWorkerStart {
    ready: Arc<Barrier>,
    start: Arc<Barrier>,
}

impl BenchmarkWorkerStart {
    pub(super) fn await_start(&self) {
        self.ready.wait();
        self.start.wait();
    }
}

/// Allocation-free completion boundary for a fixed set of benchmark workers.
pub(super) struct BenchmarkWorkerCompletionGate {
    remaining: Arc<AtomicUsize>,
    owner: Thread,
}

impl BenchmarkWorkerCompletionGate {
    pub(super) fn new(worker_count: usize) -> Self {
        assert!(worker_count > 0, "benchmark must have at least one worker");
        Self {
            remaining: Arc::new(AtomicUsize::new(worker_count)),
            owner: thread::current(),
        }
    }

    pub(super) fn worker_completion(&self) -> BenchmarkWorkerCompletion {
        BenchmarkWorkerCompletion {
            remaining: Arc::clone(&self.remaining),
            owner: self.owner.clone(),
        }
    }

    pub(super) fn wait(&self) {
        assert_eq!(
            thread::current().id(),
            self.owner.id(),
            "benchmark completion gate must be waited by its owner thread"
        );
        while self.remaining.load(Ordering::Acquire) != 0 {
            thread::park();
        }
    }
}

pub(super) struct BenchmarkWorkerCompletion {
    remaining: Arc<AtomicUsize>,
    owner: Thread,
}

impl Drop for BenchmarkWorkerCompletion {
    fn drop(&mut self) {
        if self.remaining.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.owner.unpark();
        }
    }
}

/// Immutable coordinator and build identity for one isolated benchmark process.
pub(super) struct BenchmarkRunMetadata {
    workload: &'static str,
    shape: String,
    source_manifest: String,
    cargo_profile: String,
    debug_assertions: bool,
}

impl BenchmarkRunMetadata {
    pub(super) fn from_environment(workload: &'static str, shape: String) -> Result<Self, String> {
        let source_manifest = std::env::var(SOURCE_MANIFEST_ENV)
            .map_err(|_| format!("{SOURCE_MANIFEST_ENV} must identify the materialized source"))?;
        let cargo_profile = std::env::var(CARGO_PROFILE_ENV)
            .map_err(|_| format!("{CARGO_PROFILE_ENV} must be release or profiling"))?;
        Self::new(
            workload,
            shape,
            source_manifest,
            cargo_profile,
            cfg!(debug_assertions),
        )
    }

    fn new(
        workload: &'static str,
        shape: String,
        source_manifest: String,
        cargo_profile: String,
        debug_assertions: bool,
    ) -> Result<Self, String> {
        if source_manifest.len() != 64
            || !source_manifest.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(format!(
                "{SOURCE_MANIFEST_ENV} must be a 64-digit coordinator manifest hash"
            ));
        }
        if !matches!(cargo_profile.as_str(), "release" | "profiling") {
            return Err(format!("{CARGO_PROFILE_ENV} must be release or profiling"));
        }
        if debug_assertions {
            return Err(format!(
                "benchmark profile {cargo_profile} unexpectedly enables debug assertions"
            ));
        }

        Ok(Self {
            workload,
            shape,
            source_manifest,
            cargo_profile,
            debug_assertions,
        })
    }

    /// Emit after the core interval so formatting and counter collection cannot affect it.
    pub(super) fn emit(&self, measurement: BenchmarkMeasurement<'_>) {
        let BenchmarkMeasurement {
            warmup_operations,
            measured_operations,
            elapsed,
            counters,
            latency_sample,
        } = measurement;
        let elapsed_ns = elapsed.as_nanos();
        let operations_per_second = if elapsed_ns == 0 {
            0.0
        } else {
            measured_operations as f64 * 1_000_000_000.0 / elapsed_ns as f64
        };
        let counters = counters
            .iter()
            .map(|(name, value)| format!("{}:{value}", json_string(name)))
            .collect::<Vec<_>>()
            .join(",");
        let latency = match latency_sample {
            Some(mut sample) => {
                let summary = sample.finalize();
                format!(
                    concat!(
                        "\"latency_sample_count\":{},",
                        "\"latency_p50_ns\":{},\"latency_p95_ns\":{},",
                        "\"latency_p99_ns\":{},",
                        "\"latency_percentile_algorithm\":\"nearest_rank\",",
                        "\"latency_sampling_ratio_numerator\":{},",
                        "\"latency_sampling_ratio_denominator\":{},",
                        "\"latency_observer_elapsed_ns\":{}"
                    ),
                    summary.sample_count,
                    summary.p50_ns,
                    summary.p95_ns,
                    summary.p99_ns,
                    summary.sampling_ratio_numerator,
                    summary.sampling_ratio_denominator,
                    summary.observer_elapsed.as_nanos(),
                )
            }
            None => concat!(
                "\"latency_sample_count\":0,\"latency_p50_ns\":null,",
                "\"latency_p95_ns\":null,\"latency_p99_ns\":null,",
                "\"latency_percentile_algorithm\":\"nearest_rank\",",
                "\"latency_sampling_ratio_numerator\":0,",
                "\"latency_sampling_ratio_denominator\":0,",
                "\"latency_observer_elapsed_ns\":0"
            )
            .to_owned(),
        };
        eprintln!(
            r#"{{"schema":{},"workload":{},"shape":{},"source_manifest":{},"cargo_profile":{},"debug_assertions":{},"warmup_operations":{warmup_operations},"measured_operations":{measured_operations},"elapsed_ns":{elapsed_ns},"operations_per_second":{operations_per_second:.2},{latency},"counters":{{{counters}}}}}"#,
            json_string(BENCHMARK_RECORD_SCHEMA),
            json_string(self.workload),
            json_string(&self.shape),
            json_string(&self.source_manifest),
            json_string(&self.cargo_profile),
            self.debug_assertions,
        );
    }
}

/// Values collected only after a workload's single outer timing interval has ended.
pub(super) struct BenchmarkMeasurement<'a> {
    pub(super) warmup_operations: u64,
    pub(super) measured_operations: u64,
    pub(super) elapsed: Duration,
    pub(super) counters: &'a [(&'static str, u64)],
    pub(super) latency_sample: Option<BenchmarkLatencySample<'a>>,
}

/// A bounded post-measurement sample. It is never collected in the core loop.
pub(super) struct BenchmarkLatencySample<'a> {
    pub(super) samples_ns: &'a mut [u64],
    pub(super) sampling_ratio_numerator: u64,
    pub(super) sampling_ratio_denominator: u64,
    pub(super) observer_elapsed: Duration,
}

struct BenchmarkLatencySummary {
    sample_count: usize,
    p50_ns: u64,
    p95_ns: u64,
    p99_ns: u64,
    sampling_ratio_numerator: u64,
    sampling_ratio_denominator: u64,
    observer_elapsed: Duration,
}

impl BenchmarkLatencySample<'_> {
    /// Sorting and percentile extraction are observer work, never core-workload time.
    fn finalize(&mut self) -> BenchmarkLatencySummary {
        let finalization_started = std::time::Instant::now();
        self.samples_ns.sort_unstable();
        BenchmarkLatencySummary {
            sample_count: self.samples_ns.len(),
            p50_ns: percentile(self.samples_ns, 50),
            p95_ns: percentile(self.samples_ns, 95),
            p99_ns: percentile(self.samples_ns, 99),
            sampling_ratio_numerator: self.sampling_ratio_numerator,
            sampling_ratio_denominator: self.sampling_ratio_denominator,
            observer_elapsed: self
                .observer_elapsed
                .saturating_add(finalization_started.elapsed()),
        }
    }
}

fn json_string(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len().saturating_add(2));
    encoded.push('"');
    for character in value.chars() {
        match character {
            '"' => encoded.push_str("\\\""),
            '\\' => encoded.push_str("\\\\"),
            '\u{08}' => encoded.push_str("\\b"),
            '\u{0C}' => encoded.push_str("\\f"),
            '\n' => encoded.push_str("\\n"),
            '\r' => encoded.push_str("\\r"),
            '\t' => encoded.push_str("\\t"),
            character if character <= '\u{1F}' => {
                write!(&mut encoded, "\\u{:04x}", character as u32)
                    .expect("writing a JSON escape to String cannot fail");
            }
            character => encoded.push(character),
        }
    }
    encoded.push('"');
    encoded
}

fn percentile(sorted_samples_ns: &[u64], percentile: usize) -> u64 {
    assert!(
        !sorted_samples_ns.is_empty(),
        "bounded latency samples must not be empty"
    );
    assert!(
        (1..=100).contains(&percentile),
        "latency percentile must be in the inclusive 1..=100 range"
    );
    let rank = sorted_samples_ns
        .len()
        .checked_mul(percentile)
        .expect("bounded latency percentile rank must fit usize");
    let index = rank.div_ceil(100).saturating_sub(1);
    sorted_samples_ns[index]
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE_MANIFEST: &str =
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    #[test]
    fn benchmark_metadata_accepts_source_bound_release_and_profiling_runs() {
        for cargo_profile in ["release", "profiling"] {
            let metadata = BenchmarkRunMetadata::new(
                "native_callback_atomic_lease",
                "threads=1,total_leases=1000000".to_owned(),
                SOURCE_MANIFEST.to_owned(),
                cargo_profile.to_owned(),
                false,
            )
            .expect("source-bound optimized benchmark metadata should be accepted");

            assert_eq!(metadata.source_manifest, SOURCE_MANIFEST);
            assert_eq!(metadata.cargo_profile, cargo_profile);
            assert!(!metadata.debug_assertions);
        }
    }

    #[test]
    fn benchmark_metadata_rejects_unbound_or_debug_runs() {
        let invalid_manifest = BenchmarkRunMetadata::new(
            "native_callback_atomic_lease",
            "threads=1,total_leases=1000000".to_owned(),
            "shared-worktree".to_owned(),
            "release".to_owned(),
            false,
        )
        .err()
        .expect("non-coordinator source identity must be rejected");
        assert!(invalid_manifest.contains("64-digit coordinator manifest hash"));

        let development = BenchmarkRunMetadata::new(
            "native_callback_atomic_lease",
            "threads=1,total_leases=1000000".to_owned(),
            SOURCE_MANIFEST.to_owned(),
            "development".to_owned(),
            true,
        )
        .err()
        .expect("development profile must be rejected");
        assert!(development.contains("must be release or profiling"));

        let debug_release = BenchmarkRunMetadata::new(
            "native_callback_atomic_lease",
            "threads=1,total_leases=1000000".to_owned(),
            SOURCE_MANIFEST.to_owned(),
            "release".to_owned(),
            true,
        )
        .err()
        .expect("debug release marker must be rejected");
        assert!(debug_release.contains("enables debug assertions"));
    }

    #[test]
    fn percentile_uses_bounded_sorted_samples() {
        assert_eq!(percentile(&[10, 20, 30, 40, 50], 50), 30);
        assert_eq!(percentile(&[10, 20, 30, 40, 50], 95), 50);
        assert_eq!(percentile(&[10, 20, 30, 40, 50], 99), 50);
    }

    #[test]
    fn json_string_escapes_dynamic_benchmark_metadata() {
        assert_eq!(
            json_string("native\"path\\\n\t\u{08}\u{0C}\u{1F}"),
            r#""native\"path\\\n\t\b\f\u001f""#
        );
    }

    #[test]
    fn latency_finalization_sorts_samples_and_accounts_for_sort_observer_work() {
        let mut samples = [50, 10, 30, 20, 40];
        let mut sample = BenchmarkLatencySample {
            samples_ns: &mut samples,
            sampling_ratio_numerator: 5,
            sampling_ratio_denominator: 1_000,
            observer_elapsed: Duration::from_nanos(7),
        };

        let summary = sample.finalize();

        assert_eq!(&*sample.samples_ns, &[10, 20, 30, 40, 50]);
        assert_eq!(summary.sample_count, 5);
        assert_eq!(summary.p50_ns, 30);
        assert_eq!(summary.p95_ns, 50);
        assert_eq!(summary.p99_ns, 50);
        assert_eq!(summary.sampling_ratio_numerator, 5);
        assert_eq!(summary.sampling_ratio_denominator, 1_000);
        assert!(summary.observer_elapsed >= Duration::from_nanos(7));
    }

    #[test]
    fn worker_completion_gate_releases_when_a_worker_unwinds() {
        let gate = BenchmarkWorkerCompletionGate::new(2);
        let first_completion = gate.worker_completion();
        let second_completion = gate.worker_completion();
        let first = thread::spawn(move || drop(first_completion));
        let second = thread::spawn(move || {
            let _completion = second_completion;
            panic!("expected benchmark worker failure");
        });

        gate.wait();
        first.join().expect("first completion worker");
        assert!(second.join().is_err());
        assert_eq!(gate.remaining.load(Ordering::Acquire), 0);
    }
}
