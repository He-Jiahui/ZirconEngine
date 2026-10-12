//! Original Runtime03 §9.4: authoritative Core store, all 36 metric cells.
//! This measures diagnostic workload frames; it does not certify a product frame.
use std::fs::OpenOptions;
use std::io::{self, BufWriter, Write};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Component, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use serde::Serialize;
use serde_json::{json, Value};
use windows_sys::Win32::Foundation::FILETIME;
use windows_sys::Win32::System::Threading::{GetCurrentThread, GetThreadTimes};

use super::super::collect_runtime_diagnostic_store;
use crate::core::diagnostics::{DiagnosticStore, DiagnosticStoreSnapshot};
use crate::core::{CoreHandle, CoreRuntime};

const SERIES_COUNTS: [usize; 3] = [1, 541, 10_000];
const HISTORY_LIMITS: [usize; 3] = [1, 64, 1_000];
const CONSUMER_HZ: [u64; 4] = [0, 30, 60, 120];
const REPEATS: usize = 3;
const WRITER_FRAMES: usize = 360;
const WRITER_HZ: u64 = 120;
// A cooperative stop guard, not a latency acceptance budget. An in-flight
// production mutex operation still requires the external runner death timeout.
const SAFETY_WINDOW_SECONDS: u64 = 30;
const NANOS_PER_SECOND: u64 = 1_000_000_000;
const MAX_CONTEXT_BYTES: usize = 16 * 1_024;
const MAX_CELL_BYTES: usize = 32 * 1_024 * 1_024;

#[derive(Clone, Copy, Debug, Serialize)]
struct CpuCounter {
    kernel_100ns: u64,
    user_100ns: u64,
}

impl CpuCounter {
    fn current() -> io::Result<Self> {
        let mut creation = FILETIME::default();
        let mut exit = FILETIME::default();
        let mut kernel = FILETIME::default();
        let mut user = FILETIME::default();
        // SAFETY: the current-thread pseudo handle is valid on this thread and
        // is never closed; every output is initialized and live through the call.
        let succeeded = unsafe {
            GetThreadTimes(
                GetCurrentThread(),
                &mut creation,
                &mut exit,
                &mut kernel,
                &mut user,
            )
        };
        if succeeded == 0 {
            return Err(io::Error::last_os_error());
        }
        let ticks =
            |time: FILETIME| u64::from(time.dwLowDateTime) | (u64::from(time.dwHighDateTime) << 32);
        Ok(Self {
            kernel_100ns: ticks(kernel),
            user_100ns: ticks(user),
        })
    }

    fn delta_ns(self, after: Self) -> Result<u64, String> {
        let kernel = after
            .kernel_100ns
            .checked_sub(self.kernel_100ns)
            .ok_or_else(|| "thread kernel CPU counter regressed".to_owned())?;
        let user = after
            .user_100ns
            .checked_sub(self.user_100ns)
            .ok_or_else(|| "thread user CPU counter regressed".to_owned())?;
        kernel
            .checked_add(user)
            .and_then(|ticks| ticks.checked_mul(100))
            .ok_or_else(|| "thread CPU duration overflow".to_owned())
    }
}

#[derive(Debug, Serialize)]
struct WriterSample {
    sequence: usize,
    frame_index: u64,
    requested_deadline_ns: u64,
    started_ns: u64,
    ended_ns: u64,
    service_wall_ns: u64,
    start_lateness_ns: u64,
    previous_start_interval_ns: Option<u64>,
    completed_record_calls: usize,
    cpu_before: CpuCounter,
    cpu_after: Option<CpuCounter>,
    bracket_cpu_ns: Option<u64>,
    cpu_error: Option<String>,
}

#[derive(Debug, Serialize)]
struct ReaderSample {
    sequence: usize,
    requested_slot: u64,
    requested_deadline_ns: u64,
    admitted_ns: u64,
    started_ns: u64,
    materialized_ns: u64,
    ended_ns: u64,
    materialize_wall_ns: u64,
    consume_and_drop_wall_ns: u64,
    total_wall_ns: u64,
    start_lateness_ns: u64,
    previous_start_interval_ns: Option<u64>,
    next_requested_slot: u64,
    skipped_slots_after_call: u64,
    series_count: usize,
    retained_measurements: usize,
    first_latest_frame: Option<u64>,
    last_latest_frame: Option<u64>,
    cpu_before: CpuCounter,
    cpu_after: Option<CpuCounter>,
    bracket_cpu_ns: Option<u64>,
    cpu_error: Option<String>,
}

#[derive(Debug, Serialize)]
struct WorkerRun<T> {
    status: Result<(), String>,
    errors: Vec<String>,
    cpu_before: Option<CpuCounter>,
    cpu_after: Option<CpuCounter>,
    total_thread_cpu_ns: Option<u64>,
    completed_operations: u64,
    samples: Vec<T>,
}

impl<T> WorkerRun<T> {
    fn new(capacity: usize) -> Self {
        Self {
            status: Ok(()),
            errors: Vec::new(),
            cpu_before: None,
            cpu_after: None,
            total_thread_cpu_ns: None,
            completed_operations: 0,
            samples: Vec::with_capacity(capacity),
        }
    }

    fn finish_cpu(&mut self) -> Result<(), String> {
        let after = CpuCounter::current().map_err(|error| error.to_string())?;
        self.cpu_after = Some(after);
        self.total_thread_cpu_ns = self
            .cpu_before
            .map(|before| before.delta_ns(after))
            .transpose()?;
        Ok(())
    }

    fn finish(&mut self, operation_result: Result<(), String>, epoch: Instant) {
        let cpu_result = self.finish_cpu();
        for result in [operation_result, cpu_result] {
            if let Err(error) = result {
                self.errors.push(error);
            }
        }
        if elapsed_ns(epoch, Instant::now()) >= SAFETY_WINDOW_SECONDS * NANOS_PER_SECOND {
            self.errors
                .push("worker finish exceeded cooperative safety window; raw retained".into());
        }
        self.status = self.errors.first().cloned().map_or(Ok(()), Err);
    }
}

fn finish_bracket(before: CpuCounter) -> (Option<CpuCounter>, Option<u64>, Option<String>) {
    match CpuCounter::current() {
        Ok(after) => match before.delta_ns(after) {
            Ok(delta) => (Some(after), Some(delta), None),
            Err(error) => (Some(after), None, Some(error)),
        },
        Err(error) => (None, None, Some(error.to_string())),
    }
}

#[derive(Debug, Serialize)]
struct Window {
    repeat: usize,
    writer_stop_ns: u64,
    reader_joined_ns: u64,
    writer: Option<WorkerRun<WriterSample>>,
    reader: Option<WorkerRun<ReaderSample>>,
    panic_errors: Vec<String>,
}

struct StopOnDrop<'a>(&'a AtomicBool);

impl Drop for StopOnDrop<'_> {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Release);
    }
}

fn elapsed_ns(epoch: Instant, now: Instant) -> u64 {
    u64::try_from(now.saturating_duration_since(epoch).as_nanos())
        .expect("measurement timestamp overflow")
}

fn deadline_ns(slot: u64, hz: u64) -> u64 {
    slot.checked_mul(NANOS_PER_SECOND)
        .expect("cadence deadline overflow")
        / hz
}

fn sleep_until(deadline: Instant, stop: &AtomicBool) -> bool {
    loop {
        if stop.load(Ordering::Acquire) {
            return false;
        }
        let now = Instant::now();
        if now >= deadline {
            return true;
        }
        // No busy-spin CPU is hidden in the writer/reader measurements.
        thread::sleep((deadline - now).min(Duration::from_millis(1)));
    }
}

fn value(frame_index: u64, series_index: usize) -> f64 {
    // All generated integers are exactly representable for this bounded run.
    (frame_index * 10_001 + series_index as u64) as f64
}

fn write_frame(core: &CoreHandle, paths: &[String], frame_index: u64) {
    for (series_index, path) in paths.iter().enumerate() {
        core.record_diagnostic(
            path.as_str(),
            frame_index,
            value(frame_index, series_index),
            Some("count"),
            ["runtime03", "store_matrix"],
        );
    }
}

fn run_writer(
    core: &CoreHandle,
    paths: &[String],
    first_frame: u64,
    epoch: Instant,
    stop: &AtomicBool,
) -> WorkerRun<WriterSample> {
    let mut run = WorkerRun::new(WRITER_FRAMES);
    let result = (|| -> Result<(), String> {
        run.cpu_before = Some(CpuCounter::current().map_err(|error| error.to_string())?);
        for sequence in 0..WRITER_FRAMES {
            let requested_deadline_ns = deadline_ns(sequence as u64, WRITER_HZ);
            if !sleep_until(epoch + Duration::from_nanos(requested_deadline_ns), stop) {
                return Err("writer stopped before completing its fixed frame count".into());
            }
            if Instant::now() >= epoch + Duration::from_secs(SAFETY_WINDOW_SECONDS) {
                return Err("cooperative safety window reached; matrix cell incomplete".into());
            }
            let cpu_before = CpuCounter::current().map_err(|error| error.to_string())?;
            let started = Instant::now();
            if elapsed_ns(epoch, started) >= SAFETY_WINDOW_SECONDS * NANOS_PER_SECOND {
                return Err("writer actual admission exceeded cooperative safety window".into());
            }
            let frame_index = first_frame + sequence as u64;
            write_frame(core, paths, frame_index);
            let ended = Instant::now();
            run.completed_operations += paths.len() as u64;
            let (cpu_after, bracket_cpu_ns, cpu_error) = finish_bracket(cpu_before);
            let started_ns = elapsed_ns(epoch, started);
            let previous_start_interval_ns = run
                .samples
                .last()
                .map(|previous: &WriterSample| started_ns - previous.started_ns);
            run.samples.push(WriterSample {
                sequence,
                frame_index,
                requested_deadline_ns,
                started_ns,
                ended_ns: elapsed_ns(epoch, ended),
                service_wall_ns: elapsed_ns(started, ended),
                start_lateness_ns: started_ns.saturating_sub(requested_deadline_ns),
                previous_start_interval_ns,
                completed_record_calls: paths.len(),
                cpu_before,
                cpu_after,
                bracket_cpu_ns,
                cpu_error: cpu_error.clone(),
            });
            if let Some(error) = cpu_error {
                return Err(format!(
                    "completed writer frame CPU probe failed: {error}; wall raw retained"
                ));
            }
            if elapsed_ns(epoch, ended) >= SAFETY_WINDOW_SECONDS * NANOS_PER_SECOND {
                return Err(
                    "completed writer frame exceeded cooperative safety window; raw retained"
                        .into(),
                );
            }
        }
        sleep_until(
            epoch + Duration::from_nanos(deadline_ns(WRITER_FRAMES as u64, WRITER_HZ)),
            stop,
        );
        Ok(())
    })();
    run.finish(result, epoch);
    run
}

fn run_reader(
    core: &CoreHandle,
    series_count: usize,
    hz: u64,
    epoch: Instant,
    stop: &AtomicBool,
) -> WorkerRun<ReaderSample> {
    let sample_limit = (hz * SAFETY_WINDOW_SECONDS) as usize;
    let mut run = WorkerRun::new((hz * 3 + 8) as usize);
    let result = (|| -> Result<(), String> {
        run.cpu_before = Some(CpuCounter::current().map_err(|error| error.to_string())?);
        let mut slot = 0;
        while !stop.load(Ordering::Acquire) {
            if run.samples.len() == sample_limit {
                return Err(
                    "reader raw sample safety limit reached; matrix cell incomplete".into(),
                );
            }
            let requested_deadline_ns = deadline_ns(slot, hz);
            if requested_deadline_ns >= SAFETY_WINDOW_SECONDS * NANOS_PER_SECOND {
                return Err(
                    "reader cooperative safety window reached; matrix cell incomplete".into(),
                );
            }
            if !sleep_until(epoch + Duration::from_nanos(requested_deadline_ns), stop) {
                break;
            }
            let admitted_ns = elapsed_ns(epoch, Instant::now());
            if admitted_ns >= SAFETY_WINDOW_SECONDS * NANOS_PER_SECOND {
                return Err("reader actual admission exceeded cooperative safety window".into());
            }
            if stop.load(Ordering::Acquire) {
                break;
            }
            let cpu_before = CpuCounter::current().map_err(|error| error.to_string())?;
            let started = Instant::now();
            if elapsed_ns(epoch, started) >= SAFETY_WINDOW_SECONDS * NANOS_PER_SECOND {
                return Err("reader actual start exceeded cooperative safety window".into());
            }
            let snapshot = collect_runtime_diagnostic_store(core);
            let materialized = Instant::now();
            run.completed_operations += 1;
            let observed_series = snapshot.series.len();
            let retained_measurements = snapshot
                .series
                .iter()
                .map(|entry| entry.history.len())
                .sum();
            let first_latest_frame = snapshot
                .series
                .first()
                .and_then(|entry| entry.history.last())
                .map(|measurement| measurement.frame_index);
            let last_latest_frame = snapshot
                .series
                .last()
                .and_then(|entry| entry.history.last())
                .map(|measurement| measurement.frame_index);
            std::hint::black_box(&snapshot);
            drop(snapshot);
            let ended = Instant::now();
            let (cpu_after, bracket_cpu_ns, cpu_error) = finish_bracket(cpu_before);
            let ended_ns = elapsed_ns(epoch, ended);
            // Skip missed requested periods. There is no catch-up burst, no
            // fabricated read, and no substitution of physics.fixed_hz.
            let next_requested_slot = ((u128::from(ended_ns) * u128::from(hz))
                .div_ceil(u128::from(NANOS_PER_SECOND)))
                as u64;
            let next_requested_slot = next_requested_slot.max(slot + 1);
            let started_ns = elapsed_ns(epoch, started);
            let previous_start_interval_ns = run
                .samples
                .last()
                .map(|previous: &ReaderSample| started_ns - previous.started_ns);
            let sequence = run.samples.len();
            run.samples.push(ReaderSample {
                sequence,
                requested_slot: slot,
                requested_deadline_ns,
                admitted_ns,
                started_ns,
                materialized_ns: elapsed_ns(epoch, materialized),
                ended_ns,
                materialize_wall_ns: elapsed_ns(started, materialized),
                consume_and_drop_wall_ns: elapsed_ns(materialized, ended),
                total_wall_ns: elapsed_ns(started, ended),
                start_lateness_ns: started_ns.saturating_sub(requested_deadline_ns),
                previous_start_interval_ns,
                next_requested_slot,
                skipped_slots_after_call: next_requested_slot - slot - 1,
                series_count: observed_series,
                retained_measurements,
                first_latest_frame,
                last_latest_frame,
                cpu_before,
                cpu_after,
                bracket_cpu_ns,
                cpu_error: cpu_error.clone(),
            });
            if let Some(error) = cpu_error {
                return Err(format!(
                    "completed reader snapshot CPU probe failed: {error}; wall raw retained"
                ));
            }
            if observed_series != series_count {
                return Err("actual collector changed the requested store cardinality".into());
            }
            if ended_ns >= SAFETY_WINDOW_SECONDS * NANOS_PER_SECOND {
                return Err("completed reader clone/inspect/drop exceeded cooperative safety window; raw retained".into());
            }
            slot = next_requested_slot;
        }
        Ok(())
    })();
    run.finish(result, epoch);
    run
}

fn run_window(
    core: &CoreHandle,
    paths: &[String],
    first_frame: u64,
    consumer_hz: u64,
    repeat: usize,
) -> Window {
    let stop = AtomicBool::new(false);
    let epoch = Instant::now() + Duration::from_millis(100);
    thread::scope(|scope| {
        let reader = (consumer_hz != 0)
            .then(|| scope.spawn(|| run_reader(core, paths.len(), consumer_hz, epoch, &stop)));
        let writer = catch_unwind(AssertUnwindSafe(|| {
            let _stop_on_drop = StopOnDrop(&stop);
            run_writer(core, paths, first_frame, epoch, &stop)
        }));
        let writer_stop_ns = elapsed_ns(epoch, Instant::now());
        let mut panic_errors = Vec::new();
        let writer = match writer {
            Ok(run) => Some(run),
            Err(_) => {
                panic_errors.push("writer panicked; raw evidence incomplete".into());
                None
            }
        };
        let reader = reader.and_then(|reader| match reader.join() {
            Ok(run) => Some(run),
            Err(_) => {
                panic_errors.push("reader panicked; raw evidence incomplete".into());
                None
            }
        });
        Window {
            repeat,
            writer_stop_ns,
            reader_joined_ns: elapsed_ns(epoch, Instant::now()),
            writer,
            reader,
            panic_errors,
        }
    })
}

#[derive(Debug, Serialize)]
struct Quantiles {
    sample_count: usize,
    min: u64,
    p50: u64,
    p95: u64,
    p99: u64,
    max: u64,
}

fn quantiles(values: impl Iterator<Item = u64>) -> Option<Quantiles> {
    let mut sorted: Vec<_> = values.collect();
    sorted.sort_unstable();
    if sorted.is_empty() {
        return None;
    }
    let rank = |percentile: usize| {
        let index = (sorted.len() * percentile).div_ceil(100) - 1;
        sorted[index]
    };
    Some(Quantiles {
        sample_count: sorted.len(),
        min: sorted[0],
        p50: rank(50),
        p95: rank(95),
        p99: rank(99),
        max: *sorted.last().unwrap(),
    })
}

fn window_summary(window: &Window, consumer_hz: u64) -> Value {
    let writer = window.writer.as_ref();
    let reader = window.reader.as_ref();
    let writer_samples = writer.map_or(&[][..], |run| run.samples.as_slice());
    let reader_samples = reader.map_or(&[][..], |run| run.samples.as_slice());
    let requested_slots = if consumer_hz == 0 {
        0
    } else {
        (u128::from(window.writer_stop_ns) * u128::from(consumer_hz))
            .div_ceil(u128::from(NANOS_PER_SECOND)) as u64
    };
    let actual_reads = reader.map_or(0, |run| run.completed_operations);
    let calls_started_before_stop = reader_samples
        .iter()
        .filter(|sample| sample.started_ns < window.writer_stop_ns)
        .count() as u64;
    let calls_started_after_stop = actual_reads.saturating_sub(calls_started_before_stop);
    let observed_slots = reader_samples
        .iter()
        .filter(|sample| {
            sample.requested_deadline_ns < window.writer_stop_ns
                && sample.started_ns < window.writer_stop_ns
        })
        .count() as u64;
    json!({
        "repeat": window.repeat,
        "requested_writer_hz": WRITER_HZ,
        "requested_consumer_hz": consumer_hz,
        "requested_slots_before_writer_stop": requested_slots,
        "requested_slots_serviced_before_writer_stop": observed_slots,
        "requested_slots_unserved_at_writer_stop": requested_slots.saturating_sub(observed_slots),
        "requested_slots_serviced_only_during_drain": reader_samples.iter()
            .filter(|sample| sample.requested_deadline_ns < window.writer_stop_ns
                && sample.started_ns >= window.writer_stop_ns).count(),
        "completed_consumer_calls": actual_reads,
        "reader_raw_sample_count": reader_samples.len(),
        "reader_thread_created": consumer_hz != 0,
        "reader_raw_samples_if_disabled": if consumer_hz == 0 { Some(Vec::<u64>::new()) } else { None },
        "calls_started_before_writer_stop": calls_started_before_stop,
        "calls_started_at_or_after_writer_stop": calls_started_after_stop,
        "actual_consumer_hz_before_writer_stop": calls_started_before_stop as f64 * NANOS_PER_SECOND as f64
            / window.writer_stop_ns.max(1) as f64,
        "actual_consumer_hz_including_drain": actual_reads as f64 * NANOS_PER_SECOND as f64
            / window.reader_joined_ns.max(1) as f64,
        "completed_writer_workload_batches": writer_samples.len(),
        "completed_public_record_calls": writer.map(|run| run.completed_operations),
        "actual_writer_workload_batch_hz": writer_samples.len() as f64 * NANOS_PER_SECOND as f64
            / window.writer_stop_ns.max(1) as f64,
        "reader_drain_after_writer_stop_ns": window.reader_joined_ns.saturating_sub(window.writer_stop_ns),
        "writer_frame_service_wall_ns": quantiles(writer_samples.iter().map(|sample| sample.service_wall_ns)),
        "writer_frame_start_interval_ns": quantiles(writer_samples.iter().filter_map(|sample| sample.previous_start_interval_ns)),
        "writer_frame_start_lateness_ns": quantiles(writer_samples.iter().map(|sample| sample.start_lateness_ns)),
        "writer_frame_bracket_cpu_ns": quantiles(writer_samples.iter().filter_map(|sample| sample.bracket_cpu_ns)),
        "reader_materialize_wall_ns": quantiles(reader_samples.iter().map(|sample| sample.materialize_wall_ns)),
        "reader_snapshot_consume_drop_wall_ns": quantiles(reader_samples.iter().map(|sample| sample.total_wall_ns)),
        "reader_start_interval_ns": quantiles(reader_samples.iter().filter_map(|sample| sample.previous_start_interval_ns)),
        "reader_start_lateness_ns": quantiles(reader_samples.iter().map(|sample| sample.start_lateness_ns)),
        "reader_bracket_cpu_ns": quantiles(reader_samples.iter().filter_map(|sample| sample.bracket_cpu_ns)),
        "writer_total_thread_cpu_ns": writer.and_then(|run| run.total_thread_cpu_ns),
        "reader_total_thread_cpu_ns": if consumer_hz == 0 { Some(0) } else { reader.and_then(|run| run.total_thread_cpu_ns) },
    })
}

fn audit_snapshot(
    snapshot: &DiagnosticStoreSnapshot,
    paths: &[String],
    history_limit: usize,
    total_frames: u64,
    actual_completed_calls: Option<u64>,
) -> Value {
    let mut offender_count = 0;
    let mut first_offenders = Vec::new();
    let mut reject = |message: String| {
        offender_count += 1;
        if first_offenders.len() < 32 {
            first_offenders.push(message);
        }
    };
    if snapshot.series.len() != paths.len() {
        reject(format!(
            "series cardinality {} != {}",
            snapshot.series.len(),
            paths.len()
        ));
    }
    let first_frame = total_frames.saturating_sub(history_limit as u64);
    let retained_per_series = history_limit.min(total_frames as usize);
    let mut retained_measurements = 0_u64;
    for (index, series) in snapshot.series.iter().enumerate() {
        retained_measurements += series.history.len() as u64;
        if paths
            .get(index)
            .is_none_or(|path| path != series.path.as_str())
        {
            reject(format!("series {index} path identity differs"));
        }
        if series.unit.as_deref() != Some("count")
            || series.subsystem_tags != ["runtime03", "store_matrix"]
        {
            reject(format!("series {index} metadata differs"));
        }
        if series.history.len() != retained_per_series {
            reject(format!("series {index} history length differs"));
        }
        for (offset, measurement) in series.history.iter().enumerate() {
            let expected_frame = first_frame + offset as u64;
            if measurement.frame_index != expected_frame
                || measurement.value != value(expected_frame, index)
            {
                reject(format!(
                    "series {index} history[{offset}] frame/value differs"
                ));
            }
        }
        let expected_current = value(total_frames - 1, index);
        let expected_ema = (1..total_frames).fold(value(0, index), |previous, frame| {
            previous.mul_add(0.9, value(frame, index) * 0.1)
        });
        if series.current != Some(expected_current)
            || series.min != Some(value(0, index))
            || series.max != Some(expected_current)
            || series.smoothed != Some(expected_ema)
        {
            reject(format!(
                "series {index} current/lifetime extrema/exact EMA recurrence differs"
            ));
        }
    }
    let expected_completed_calls = total_frames * paths.len() as u64;
    let expected_retained = retained_per_series as u64 * paths.len() as u64;
    if retained_measurements != expected_retained {
        reject("total retained cardinality differs".into());
    }
    json!({
        "audit_scope": "outside measured windows; authoritative CoreHandle snapshot, not a consumer call",
        "expected_completed_record_calls": expected_completed_calls,
        "actual_completed_record_calls": actual_completed_calls,
        "retained_measurements": retained_measurements,
        "inferred_overwritten_measurements": actual_completed_calls
            .and_then(|calls| calls.checked_sub(retained_measurements)),
        "production_drop_counter": null,
        "first_retained_frame": first_frame,
        "last_retained_frame": total_frames - 1,
        "all_history_entries_checked": true,
        "offender_count": offender_count,
        "first_offenders": first_offenders,
        "conservation": actual_completed_calls.map(|calls| {
            calls == expected_completed_calls && retained_measurements == expected_retained
        }),
        "overwrite_semantics": "inferred from completed public calls and Source record-always-appends; not a production drop counter",
    })
}

fn checked_output_path() -> io::Result<PathBuf> {
    let requested = std::env::var_os("ZR_RUNTIME03_STORE_MATRIX_OUTPUT")
        .map(PathBuf::from)
        .ok_or_else(|| {
            io::Error::other("registered ZR_RUNTIME03_STORE_MATRIX_OUTPUT is required")
        })?;
    if !requested.is_absolute()
        || requested
            .components()
            .any(|part| matches!(part, Component::ParentDir))
    {
        return Err(io::Error::other(
            "output must be an absolute path without parent traversal",
        ));
    }
    let parent = requested
        .parent()
        .ok_or_else(|| io::Error::other("missing output parent"))?;
    let name = requested
        .file_name()
        .ok_or_else(|| io::Error::other("missing output file name"))?;
    let physical_parent = parent.canonicalize()?;
    for drive in ["D", "E", "F"] {
        let base = PathBuf::from(format!("{drive}:/cargo-targets/zircon-local"));
        if !requested.starts_with(&base) {
            continue;
        }
        let physical_base = base.canonicalize()?;
        let expected_base = PathBuf::from(format!(r"\\?\{drive}:\cargo-targets\zircon-local"));
        let original_relative_parent = parent
            .strip_prefix(&base)
            .map_err(|error| io::Error::other(error.to_string()))?;
        let expected_parent = expected_base.join(original_relative_parent);
        if physical_base == expected_base
            && physical_parent == expected_parent
            && physical_parent.starts_with(&physical_base)
        {
            return Ok(physical_parent.join(name));
        }
    }
    Err(io::Error::other(
        "output must physically remain in drive-root D/E/F:/cargo-targets/zircon-local",
    ))
}

fn write_line(output: &mut impl Write, value: &Value) -> io::Result<()> {
    serde_json::to_writer(&mut *output, value)?;
    output.write_all(b"\n")?;
    output.flush()
}

fn shutdown_receipt(runtime: &CoreRuntime) -> (bool, Value) {
    let result = runtime.shutdown_task_graph(Duration::from_secs(SAFETY_WINDOW_SECONDS));
    let (api_ok, report, error) = match result {
        Ok(report) => (true, report, None),
        Err(error) => {
            let message = error.to_string();
            (false, error.report, Some(message))
        }
    };
    let complete = api_ok
        && !report.has_in_flight_work()
        && report
            .worker_shutdowns
            .iter()
            .all(|workers| workers.all_joined());
    let workers: Vec<_> = report
        .worker_shutdowns
        .iter()
        .map(|workers| {
            json!({
                "kind": format!("{:?}", workers.kind),
                "active_submission_count": workers.active_submission_count,
                "expected_worker_count": workers.expected_worker_count,
                "exited_worker_count": workers.exited_worker_count,
                "joined_worker_count": workers.joined_worker_count,
                "termination_signalled": workers.termination_signalled,
                "all_joined": workers.all_joined(),
            })
        })
        .collect();
    (
        complete,
        json!({
            "scope": "outside all writer/reader windows and authority audits",
            "api": "CoreRuntime::shutdown_task_graph",
            "operational_timeout_seconds": SAFETY_WINDOW_SECONDS,
            "api_returned_ok": api_ok,
            "complete": complete,
            "error": error,
            "elapsed_ns": u64::try_from(report.elapsed.as_nanos()).expect("shutdown timestamp overflow"),
            "timer_started": report.timer_started,
            "timer_joined": report.timer_joined,
            "scope_count": report.scopes.len(),
            "scopes_quiescent": report.scopes.iter().all(|scope| scope.is_quiescent()),
            "workers": workers,
            "whole_runtime_teardown": false,
        }),
    )
}

fn source_fingerprints() -> Value {
    let entries: [(&str, &[u8]); 18] = [
        (
            "store",
            include_bytes!("../../core/runtime/diagnostics/store.rs"),
        ),
        (
            "core_handle_diagnostics",
            include_bytes!("../../core/runtime/handle/diagnostics.rs"),
        ),
        (
            "core_handle",
            include_bytes!("../../core/runtime/handle/core_handle.rs"),
        ),
        (
            "core_runtime",
            include_bytes!("../../core/runtime/runtime.rs"),
        ),
        (
            "core_owner",
            include_bytes!("../../core/runtime/state/core_runtime_state.rs"),
        ),
        ("collector", include_bytes!("../collect.rs")),
        (
            "physics_enabled",
            include_bytes!("../physics_collection_enabled.rs"),
        ),
        (
            "physics_disabled",
            include_bytes!("../physics_collection_disabled.rs"),
        ),
        (
            "existing_tests",
            include_bytes!("collect_performance_tests.rs"),
        ),
        ("producer", include_bytes!("original_store_matrix.rs")),
        (
            "engine_task_graph",
            include_bytes!("../../core/runtime/tasks/task_graph/engine_task_graph.rs"),
        ),
        (
            "task_graph_shutdown",
            include_bytes!("../../core/runtime/tasks/task_graph/shutdown.rs"),
        ),
        (
            "task_graph_scope_census",
            include_bytes!("../../core/runtime/tasks/task_graph/scope_model.rs"),
        ),
        (
            "task_graph_worker_census",
            include_bytes!("../../core/runtime/tasks/task_graph/worker_inventory.rs"),
        ),
        (
            "task_pool",
            include_bytes!("../../core/runtime/tasks/pool.rs"),
        ),
        (
            "task_pools",
            include_bytes!("../../core/runtime/tasks/pools.rs"),
        ),
        (
            "owned_worker_join",
            include_bytes!("../../core/runtime/tasks/pool/owned_workers.rs"),
        ),
        (
            "native_thread_join_readiness",
            include_bytes!("../../core/runtime/tasks/thread_completion.rs"),
        ),
    ];
    json!(entries
        .into_iter()
        .map(|(name, bytes)| json!({"name":name,"algorithm":"blake3","digest":blake3::hash(bytes).to_hex().to_string()}))
        .collect::<Vec<_>>())
}

#[test]
#[ignore = "original Runtime03 real Core store matrix; Release, registered output, exclusive process"]
fn runtime03_original_diagnostic_store_series_history_consumer_matrix() {
    assert!(!cfg!(debug_assertions), "run this producer in Release");
    let context_text = std::env::var("ZR_RUNTIME03_STORE_MATRIX_CONTEXT_JSON")
        .expect("hash-bound host/profile/cpu/gpu/driver context is required");
    assert!(
        context_text.len() <= MAX_CONTEXT_BYTES,
        "external context exceeds safety bound"
    );
    let context: Value = serde_json::from_str(&context_text).expect("valid matrix context JSON");
    for key in [
        "profile",
        "cpu",
        "gpu",
        "driver",
        "run_id",
        "runner_death_timeout_seconds",
    ] {
        assert!(
            context.get(key).is_some(),
            "missing external context: {key}"
        );
    }
    assert_eq!(context["profile"], "release");
    assert!(context["runner_death_timeout_seconds"]
        .as_u64()
        .is_some_and(|value| value > 0));
    let path = checked_output_path().expect("pre-registered physical evidence output");
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .expect("new evidence output; never overwrite a receipt");
    let mut output = BufWriter::new(file);
    write_line(&mut output, &json!({
        "record": "environment",
        "schema": "RUNTIME03_REAL_CORE_STORE_ORIGINAL_MATRIX_V1",
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "processor_identifier": std::env::var("PROCESSOR_IDENTIFIER").ok(),
        "available_parallelism": thread::available_parallelism().map(|count| count.get()).ok(),
        "external_context": context,
        "source_fingerprints": source_fingerprints(),
        "series_counts": SERIES_COUNTS,
        "history_limits": HISTORY_LIMITS,
        "consumer_hz": CONSUMER_HZ,
        "expected_cells": 36,
        "repeats_per_cell": REPEATS,
        "max_raw_writer_samples_per_cell": REPEATS * WRITER_FRAMES,
        "max_raw_reader_samples_per_cell": REPEATS as u64 * 120 * SAFETY_WINDOW_SECONDS,
        "max_serialized_cell_bytes": MAX_CELL_BYTES,
        "max_external_context_bytes": MAX_CONTEXT_BYTES,
        "largest_authority_history_measurements": 10_000 * 1_000,
        "measurement_payload_bytes": std::mem::size_of::<crate::core::diagnostics::DiagnosticMeasurement>(),
        "largest_live_history_payload_copies": 2,
        "live_payload_limit_note": "one authoritative history plus one cloned full snapshot; allocator/metadata/raw bookkeeping add bytes",
        "fixed_writer_frames_per_repeat": WRITER_FRAMES,
        "requested_writer_hz": WRITER_HZ,
        "cooperative_safety_window_seconds": SAFETY_WINDOW_SECONDS,
        "cpu_unit": "GetThreadTimes 100ns kernel/user counters; nanosecond deltas retain quantized zeros",
        "cpu_probe_failure": "completed wall/slot/operation raw is retained with nullable CPU after/delta and explicit error; worker records all operation/finish/safety errors",
        "cpu_scope": "writer/reader own thread; brackets include counter instrumentation; totals also include pacing/bookkeeping; runtime graph worker CPU is excluded",
        "wall_scope": "public write batch; actual full collector materialization and output consume/drop",
        "clock": "Instant monotonic; no busy-spin; OS sleep lateness and missed requested slots are raw evidence",
        "safety_window": "actual admission/start/end/finish checked; in-flight operations finish and retain raw, then crossing marks incomplete; external process death timeout remains required",
        "frame_scope": "controlled diagnostic workload, not engine simulation/render/product frames",
        "noise_protocol": "3 sequential repetitions; retain all raw data and observed repeat quantiles, no invented confidence interval",
        "quantile_precision": "nearest-rank order statistic with raw sample count; overloaded small n is disclosed, not certified stable p99",
        "snapshot_frame_consistency": "one production authority snapshot; writer series are independently locked and may expose adjacent partial workload frames",
        "budget": null,
        "product_acceptance": false,
        "current_snapshot_matrix": "not measured",
        "M1_M2_M3_solved": false,
        "cell_worker_lifecycle": "existing real shutdown_task_graph report must prove owned domains joined before starting the next cell; Drop alone is not join evidence",
        "teardown_failure": "write current row and incomplete footer; remaining requested matrix cells are explicitly not run; external death timeout still required",
        "completion": "only the final 36-cell footer may establish protocol completion",
        "fatal_paths": "previously flushed cells remain; current cell can be unknown after Core setup, warmup, authority audit, OOM, output or final Drop failure; missing completion footer never passes",
    })).unwrap();

    let mut cells = 0;
    let mut failures = Vec::new();
    let mut raw_hasher = blake3::Hasher::new();
    let mut retained_closing_owner = None;
    'matrix: for series_count in SERIES_COUNTS {
        for history_limit in HISTORY_LIMITS {
            for consumer_hz in CONSUMER_HZ {
                let runtime = CoreRuntime::new();
                let core = runtime.handle();
                // This changes the existing production owner's authoritative store.
                // No replacement Arc<Mutex<DiagnosticStore>>, public shim or fake owner.
                core.update_diagnostic_store(|store| *store = DiagnosticStore::new(history_limit));
                let paths: Vec<_> = (0..series_count)
                    .map(|index| format!("runtime03.original_store.{index:05}"))
                    .collect();
                let warmup_started = Instant::now();
                for frame_index in 0..history_limit as u64 {
                    write_frame(&core, &paths, frame_index);
                }
                let warmup_wall_ns = elapsed_ns(warmup_started, Instant::now());
                let warmup_audit = audit_snapshot(
                    &core.diagnostic_store_snapshot(),
                    &paths,
                    history_limit,
                    history_limit as u64,
                    Some((series_count * history_limit) as u64),
                );
                let mut windows = Vec::with_capacity(REPEATS);
                for repeat in 0..REPEATS {
                    windows.push(run_window(
                        &core,
                        &paths,
                        history_limit as u64 + (repeat * WRITER_FRAMES) as u64,
                        consumer_hz,
                        repeat,
                    ));
                }
                let total_frames = history_limit as u64 + (REPEATS * WRITER_FRAMES) as u64;
                let actual_completed_calls = windows.iter().try_fold(
                    (series_count * history_limit) as u64,
                    |calls, window| {
                        window
                            .writer
                            .as_ref()
                            .map(|run| calls + run.completed_operations)
                    },
                );
                let final_audit = audit_snapshot(
                    &core.diagnostic_store_snapshot(),
                    &paths,
                    history_limit,
                    total_frames,
                    actual_completed_calls,
                );
                let (teardown_complete, task_graph_shutdown) = shutdown_receipt(&runtime);
                let complete = teardown_complete
                    && warmup_audit["offender_count"] == 0
                    && final_audit["offender_count"] == 0
                    && final_audit["conservation"] == true
                    && windows.iter().all(|window| {
                        window.panic_errors.is_empty()
                            && window.writer.as_ref().is_some_and(|run| {
                                run.status.is_ok()
                                    && run.samples.len() == WRITER_FRAMES
                                    && run.completed_operations
                                        == (series_count * WRITER_FRAMES) as u64
                            })
                            && if consumer_hz == 0 {
                                window.reader.is_none()
                            } else {
                                window.reader.as_ref().is_some_and(|run| {
                                    run.status.is_ok()
                                        && !run.samples.is_empty()
                                        && run.completed_operations == run.samples.len() as u64
                                        && run.samples.iter().all(|sample| {
                                            sample.series_count == series_count
                                                && sample.retained_measurements
                                                    == series_count * history_limit
                                        })
                                })
                            }
                    });
                let summaries: Vec<_> = windows
                    .iter()
                    .map(|window| window_summary(window, consumer_hz))
                    .collect();
                let pooled_writer_wall = quantiles(
                    windows
                        .iter()
                        .filter_map(|window| window.writer.as_ref())
                        .flat_map(|run| run.samples.iter().map(|sample| sample.service_wall_ns)),
                );
                let pooled_writer_cpu = quantiles(
                    windows
                        .iter()
                        .filter_map(|window| window.writer.as_ref())
                        .flat_map(|run| {
                            run.samples
                                .iter()
                                .filter_map(|sample| sample.bracket_cpu_ns)
                        }),
                );
                let pooled_reader_wall = quantiles(
                    windows
                        .iter()
                        .filter_map(|window| window.reader.as_ref())
                        .flat_map(|run| run.samples.iter().map(|sample| sample.total_wall_ns)),
                );
                let pooled_reader_cpu = quantiles(
                    windows
                        .iter()
                        .filter_map(|window| window.reader.as_ref())
                        .flat_map(|run| {
                            run.samples
                                .iter()
                                .filter_map(|sample| sample.bracket_cpu_ns)
                        }),
                );
                let cell = json!({
                    "record": "matrix_cell",
                    "series_count": series_count, "history_limit": history_limit,
                    "requested_consumer_hz": consumer_hz, "protocol_complete": complete,
                    "default_core_worker_inventory": format!("{:?}", runtime.task_graph_worker_inventory()),
                    "real_task_graph_shutdown": task_graph_shutdown,
                    "warmup_completed_record_calls": series_count * history_limit,
                    "warmup_wall_ns": warmup_wall_ns,
                    "boundary_authority_audits": 2,
                    "zero_hz_consumer_calls": if consumer_hz == 0 { Some(0_u64) } else { None },
                    "warmup_audit": warmup_audit, "final_audit": final_audit,
                    "writer_frame_pooled_service_wall_ns": pooled_writer_wall,
                    "writer_frame_pooled_bracket_cpu_ns": pooled_writer_cpu,
                    "reader_pooled_snapshot_consume_drop_wall_ns": pooled_reader_wall,
                    "reader_pooled_bracket_cpu_ns": pooled_reader_cpu,
                    "repeat_quantiles_and_cadence": summaries,
                    "raw_windows": windows,
                });
                let serialized = serde_json::to_vec(&cell).unwrap();
                assert!(serialized.len() <= MAX_CELL_BYTES,
                    "raw cell exceeds safety bound; retain incomplete file, never truncate and pass");
                raw_hasher.update(&serialized);
                raw_hasher.update(b"\n");
                output.write_all(&serialized).unwrap();
                output.write_all(b"\n").unwrap();
                output.flush().unwrap();
                cells += 1;
                if !complete {
                    failures.push(json!({
                        "series_count":series_count,"history_limit":history_limit,
                        "consumer_hz":consumer_hz,"reason":"raw cell or authority conservation incomplete",
                    }));
                }
                if !teardown_complete {
                    // Retain the closing production owner while its failure/footer
                    // evidence is written. No subsequent Core or row is constructed.
                    retained_closing_owner = Some(runtime);
                    break 'matrix;
                }
                // A successful existing shutdown receipt proves physical owned
                // worker joins before the next row, beyond mere Core/graph Drop.
            }
        }
    }
    write_line(
        &mut output,
        &json!({
            "record": "completion",
            "attempted_cells": cells,
            "flushed_cells": cells,
            "protocol_complete_cells": cells - failures.len(),
            "not_run_cells": 36 - cells,
            "stopped_after_task_graph_shutdown_failure": retained_closing_owner.is_some(),
            "cells": cells,
            "expected_cells": 36,
            "complete": cells == 36 && failures.is_empty(),
            "failures": failures,
            "matrix_lines_blake3": raw_hasher.finalize().to_hex().to_string(),
            "numeric_latency_acceptance": "no absolute budget in original §9.4",
            "native_gain_product_acceptance": false,
        }),
    )
    .unwrap();
    println!(
        "RUNTIME03_REAL_CORE_STORE_ORIGINAL_MATRIX_V1 cells={cells} output={} complete={}",
        path.display(),
        cells == 36 && failures.is_empty()
    );
    // This owner remains alive until the raw incomplete footer is flushed.
    // Physical process death, including any blocking final Drop, is a runner gate.
    let _retained_closing_owner = retained_closing_owner;
    assert_eq!(cells, 36);
    assert!(
        failures.is_empty(),
        "matrix protocol incomplete; keep raw failed evidence"
    );
}
