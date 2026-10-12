//! Original Runtime03 config payload/key-burst matrix over the actual config worker.
//! The probe labels its overhead and does not establish a product performance budget.
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::os::windows::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::{json, Value};

use super::super::{
    recover_and_load_from_disk, ConfigCommitFence, ConfigFileWriter, DefaultConfigManager,
    DEFAULT_PERSISTENCE_DEBOUNCE, DEFAULT_SHUTDOWN_FLUSH_TIMEOUT,
};
use super::{ConfigPersistenceShared, ConfigPersistenceWorker, ConfigSnapshotSource};
use crate::core::framework::foundation::{ConfigManager, ConfigPersistenceReport};
use crate::core::resource::io::stage_atomic_write;
use crate::core::CoreRuntime;

const PAYLOAD_BYTES: [usize; 3] = [1, 1_048_576, 104_857_600];
const DISTINCT_KEYS: [usize; 2] = [1, 1_000];
const SAMPLES: usize = 5;
const FLUSH_SAFETY: Duration = Duration::from_secs(30);
const OUTPUT: &str =
    r"E:\cargo-targets\zircon-local\reports\runtime-editor-optimize\runtime03-config-matrix-v1";

#[derive(Clone, Copy)]
struct SnapshotTiming {
    started_ns: u64,
    finished_ns: u64,
}

struct WriteTiming {
    snapshot: SnapshotTiming,
    writer_entered_ns: u64,
    stage_finished_ns: u64,
    commit_finished_ns: u64,
    serialized_bytes: usize,
    commit_attempted: bool,
    caller_thread: bool,
    error: Option<String>,
}

struct Probe {
    origin: Instant,
    caller: thread::ThreadId,
    last_snapshot: Option<SnapshotTiming>,
    writes: Vec<WriteTiming>,
}

struct TimingAtomicWriter(Arc<Mutex<Probe>>);

impl ConfigFileWriter for TimingAtomicWriter {
    fn write(&self, path: &Path, bytes: &[u8], commit_fence: &ConfigCommitFence) -> io::Result<()> {
        let (origin, caller, snapshot) = {
            let probe = self.0.lock().expect("config matrix probe");
            (probe.origin, probe.caller, probe.last_snapshot)
        };
        let writer_entered_ns = elapsed_ns(origin);
        let snapshot = snapshot.ok_or_else(|| io::Error::other("missing real snapshot probe"))?;
        // These are the exact production AtomicConfigFileWriter stage/fence/commit calls.
        let pending = stage_atomic_write(path, bytes);
        let stage_finished_ns = elapsed_ns(origin);
        let commit_attempted = pending.is_ok();
        let result = pending.and_then(|pending| commit_fence.commit(|| pending.commit()));
        let commit_finished_ns = elapsed_ns(origin);
        self.0
            .lock()
            .expect("config matrix probe")
            .writes
            .push(WriteTiming {
                snapshot,
                writer_entered_ns,
                stage_finished_ns,
                commit_finished_ns,
                serialized_bytes: bytes.len(),
                commit_attempted,
                caller_thread: thread::current().id() == caller,
                error: result.as_ref().err().map(ToString::to_string),
            });
        result
    }
}

#[test]
#[ignore = "Windows Release original Runtime03 config matrix; bounded coordinator execution"]
fn runtime03_original_config_persistence_payload_key_burst_matrix() {
    assert!(!cfg!(debug_assertions), "Release execution is required");
    let output = checked_output().expect("registered physical config matrix output");
    let context: Value = serde_json::from_slice(
        &fs::read(output.join("context.json")).expect("source/toolchain/hardware context"),
    )
    .expect("valid config matrix context");
    assert_eq!(context["task_id"], "runtime-editor-optimize");
    assert_eq!(context["output_directory"], OUTPUT);
    for field in [
        "request_id",
        "source_sha256",
        "toolchain",
        "cpu",
        "gpu",
        "driver",
        "profile",
    ] {
        assert!(
            context.get(field).is_some_and(|v| !v.is_null()),
            "missing context {field}"
        );
    }
    assert_eq!(context["profile"], "release");
    assert!(context["external_process_deadline_ms"]
        .as_u64()
        .is_some_and(|v| v > 0));
    // The context records requested supervision; only a closed coordinator receipt proves it.
    let mut log = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output.join("raw-samples.jsonl"))
        .expect("exclusive raw output");
    emit(&mut log, &json!({
        "marker": "RUNTIME03_CONFIG_MATRIX_CONTEXT_V1", "context": context,
        "requested_payload_bytes": PAYLOAD_BYTES, "distinct_burst_keys": DISTINCT_KEYS,
        "samples_per_shape_per_completion": SAMPLES, "planned_shapes": 6,
        "planned_samples": 60, "debounce_ms": DEFAULT_PERSISTENCE_DEBOUNCE.as_millis(),
        "shutdown_wait_budget_ms": DEFAULT_SHUTDOWN_FLUSH_TIMEOUT.as_millis(),
        "flush_safety_ms": FLUSH_SAFETY.as_millis(),
        "numeric_performance_budget": null, "whole_runtime03_accepted": false,
        "probe_overhead": "snapshot closure and stage/fence observer; worker serialized interval includes return/dispatch overhead",
        "serializer_probe": "independent exact serde_json::to_vec_pretty over fresh actual Core snapshot outside persistence measurement",
        "payload_size": "one ASCII string payload; complete JSON also contains keys/numbers/formatting"
    })).expect("context output");
    let fixtures = output.join("fixtures");
    fs::create_dir(&fixtures).expect("exclusive owned fixture directory");
    let mut completed = 0;
    let mut failure = None;
    let mut closing_owner = None;
    'matrix: for payload in PAYLOAD_BYTES {
        for keys in DISTINCT_KEYS {
            let path = fixtures.join(format!("payload-{payload}-keys-{keys}.json"));
            match run_shape(&path, payload, keys, &mut log, &mut closing_owner) {
                Ok(()) => completed += 1,
                Err(error) => {
                    failure = Some(error);
                    break 'matrix;
                }
            }
        }
    }
    emit(&mut log, &json!({
        "marker": "RUNTIME03_CONFIG_MATRIX_TERMINAL_V1", "complete": failure.is_none() && completed == 6,
        "completed_shapes": completed, "not_completed_shapes": 6-completed, "error": failure,
        "failed_graph_owner_retained_through_footer": closing_owner.is_some(),
        "product_performance_accepted": false, "whole_runtime03_accepted": false,
        "fixture_cleanup": "retained for hash-bound review after worker and task-graph exit; no automatic deletion"
    })).expect("terminal output");
    log.sync_all().expect("durable config matrix raw output");
    drop(closing_owner);
    println!("RUNTIME03_CONFIG_MATRIX_TERMINAL_V1 complete={} completed_shapes={completed} planned_shapes=6 planned_samples=60",
        failure.is_none() && completed == 6);
    assert!(
        failure.is_none() && completed == 6,
        "incomplete config matrix: {failure:?}"
    );
}

fn run_shape(
    path: &Path,
    payload: usize,
    count: usize,
    log: &mut File,
    closing_owner: &mut Option<CoreRuntime>,
) -> Result<(), String> {
    let keys: Vec<String> = (0..count)
        .map(|i| format!("runtime03.burst.{i:04}"))
        .collect();
    run_flush_session(path, payload, &keys, log, closing_owner)?;
    for sample in 0..SAMPLES {
        run_shutdown_sample(path, payload, &keys, sample, log, closing_owner)?;
    }
    println!("RUNTIME03_CONFIG_MATRIX_SHAPE_V1 payload_bytes={payload} distinct_keys={count} flush_samples={SAMPLES} shutdown_samples={SAMPLES} numeric_budget=unassigned");
    Ok(())
}

fn run_flush_session(
    path: &Path,
    payload: usize,
    keys: &[String],
    log: &mut File,
    closing_owner: &mut Option<CoreRuntime>,
) -> Result<(), String> {
    let runtime = CoreRuntime::new();
    {
        let core = runtime.handle();
        core.store_config_value("runtime03.payload", Value::String("x".repeat(payload)));
        for key in keys {
            core.store_config_value(key, Value::from(0));
        }
    }
    let mut config_exited = None;
    let mut final_report = None;
    let session_result = (|| {
        let (manager, probe, observed) = measured_manager(&runtime, path)?;
        let measured = (|| {
            manager
                .set_value(&keys[0], Value::from(1))
                .map_err(|e| e.to_string())?;
            manager.flush(FLUSH_SAFETY).map_err(|e| e.to_string())?;
            for sample in 0..SAMPLES {
                probe.lock().expect("config matrix probe").writes.clear();
                let before = manager.persistence_report();
                let begun = Instant::now();
                let mut errors = Vec::new();
                for key in keys {
                    if let Err(error) = manager.set_value(key, Value::from(sample + 2)) {
                        errors.push(error.to_string());
                        break;
                    }
                }
                let burst_ns = ns(begun.elapsed());
                let flush_started = Instant::now();
                let flush = manager.flush(FLUSH_SAFETY);
                let completion_ns = ns(flush_started.elapsed());
                if let Err(error) = flush {
                    errors.push(error.to_string());
                }
                let after = manager.persistence_report();
                // A partial burst/flush error still emits completed timing and report evidence.
                record_sample(
                    log,
                    path,
                    payload,
                    keys,
                    sample,
                    "flush",
                    sample + 2,
                    burst_ns,
                    completion_ns,
                    &before,
                    &after,
                    &probe,
                    errors,
                    &runtime,
                    false,
                )?;
            }
            Ok::<(), String>(())
        })();
        // This is the only strong worker owner; shared observation does not retain it.
        drop(manager);
        {
            let state = observed.lock_state();
            config_exited = Some(state.worker_exited);
            final_report = Some(state.report());
        }
        drop(observed);
        drop(probe);
        measured
    })();
    finish_session(
        runtime,
        log,
        "flush-session",
        payload,
        keys.len(),
        config_exited,
        final_report,
        session_result,
        closing_owner,
    )
}

fn run_shutdown_sample(
    path: &Path,
    payload: usize,
    keys: &[String],
    sample: usize,
    log: &mut File,
    closing_owner: &mut Option<CoreRuntime>,
) -> Result<(), String> {
    let runtime = CoreRuntime::new();
    let mut config_exited = None;
    let mut final_report = None;
    let session_result = (|| {
        // Actual recovery reloads the preceding committed complete JSON.
        let (manager, probe, observed) = measured_manager(&runtime, path)?;
        let before = manager.persistence_report();
        let begun = Instant::now();
        let value = sample + 100;
        let mut errors = Vec::new();
        for key in keys {
            if let Err(error) = manager.set_value(key, Value::from(value)) {
                errors.push(error.to_string());
                break;
            }
        }
        let burst_ns = ns(begun.elapsed());
        let completion_started = Instant::now();
        drop(manager);
        let completion_ns = ns(completion_started.elapsed());
        let (after, exited) = {
            let state = observed.lock_state();
            (state.report(), state.worker_exited)
        };
        config_exited = Some(exited);
        let measured = record_sample(
            log,
            path,
            payload,
            keys,
            sample,
            "last_owner_shutdown",
            value,
            burst_ns,
            completion_ns,
            &before,
            &after,
            &probe,
            errors,
            &runtime,
            exited,
        );
        // Preserve actual last_error even when the production serializer did not enter write.
        final_report = Some(observed.lock_state().report());
        // The snapshot closure owns the ConfigStore, so release shared state before the next Core.
        drop(observed);
        drop(probe);
        measured
    })();
    finish_session(
        runtime,
        log,
        "shutdown-session",
        payload,
        keys.len(),
        config_exited,
        final_report,
        session_result,
        closing_owner,
    )
}

// Always attempt the real graph join, including constructor, warmup, measurement and audit errors.
fn finish_session(
    runtime: CoreRuntime,
    log: &mut File,
    completion: &str,
    payload: usize,
    keys: usize,
    config_exited: Option<bool>,
    final_report: Option<ConfigPersistenceReport>,
    session_result: Result<(), String>,
    closing_owner: &mut Option<CoreRuntime>,
) -> Result<(), String> {
    let shutdown = runtime.shutdown_task_graph(FLUSH_SAFETY);
    let report = match &shutdown {
        Ok(r) => r,
        Err(e) => &e.report,
    };
    let joined = shutdown.is_ok() && !report.has_in_flight_work();
    let owner_ok = config_exited != Some(false) && joined;
    let recorded = emit(log,&json!({
        "marker":"RUNTIME03_CONFIG_MATRIX_OWNER_EXIT_V1", "completion":completion,
        "payload_bytes":payload, "distinct_keys":keys, "config_worker_exited":config_exited,
        "config_worker_started":config_exited.is_some(), "graph_elapsed_ns":ns(report.elapsed),
        "graph_timer_joined":report.timer_joined, "graph_has_in_flight_work":report.has_in_flight_work(),
        "graph_worker_census":format!("{:?}",report.worker_shutdowns),
        "graph_error":shutdown.as_ref().err().map(ToString::to_string),
        "session_error":session_result.as_ref().err(), "owner_joined":owner_ok,
        "final_config_persistence_report":final_report.as_ref().map(|r|json!({
            "dirty_generation":r.dirty_generation,"persisted_generation":r.persisted_generation,
            "attempted_flushes":r.flush_attempts,"successful_writes":r.successful_writes,
            "failed_writes":r.failed_writes,"serialized_bytes":r.serialized_bytes,
            "pending_flushes":r.pending_flushes,"last_error":r.last_error
        }))
    })).map_err(|e|e.to_string());
    if joined {
        drop(runtime);
    } else {
        *closing_owner = Some(runtime);
    }
    recorded?;
    session_result?;
    if owner_ok {
        Ok(())
    } else {
        Err("config worker or real task graph did not exit/join".into())
    }
}

fn measured_manager(
    runtime: &CoreRuntime,
    path: &Path,
) -> Result<
    (
        DefaultConfigManager,
        Arc<Mutex<Probe>>,
        Arc<ConfigPersistenceShared>,
    ),
    String,
> {
    let core = runtime.handle();
    let fence = ConfigCommitFence::register(path).map_err(|e| e.to_string())?;
    recover_and_load_from_disk(&core, &path.to_path_buf(), &fence).map_err(|e| e.to_string())?;
    let probe = Arc::new(Mutex::new(Probe {
        origin: Instant::now(),
        caller: thread::current().id(),
        last_snapshot: None,
        writes: Vec::new(),
    }));
    let actual_source = core.config_snapshot_source();
    let for_snapshot = Arc::clone(&probe);
    let snapshot: ConfigSnapshotSource = Arc::new(move || {
        let origin = for_snapshot.lock().expect("config matrix probe").origin;
        let started_ns = elapsed_ns(origin);
        let values = actual_source();
        let finished_ns = elapsed_ns(origin);
        for_snapshot
            .lock()
            .expect("config matrix probe")
            .last_snapshot = Some(SnapshotTiming {
            started_ns,
            finished_ns,
        });
        values
    });
    let path = Arc::new(path.to_path_buf());
    let worker = ConfigPersistenceWorker::start(
        Arc::clone(&path),
        snapshot,
        Arc::new(TimingAtomicWriter(Arc::clone(&probe))),
        DEFAULT_PERSISTENCE_DEBOUNCE,
        DEFAULT_SHUTDOWN_FLUSH_TIMEOUT,
        fence,
    )
    .map_err(|e| e.to_string())?;
    let observed = Arc::clone(&worker.shared);
    let manager = DefaultConfigManager {
        core: core.downgrade(),
        path,
        persistence: worker,
    };
    Ok((manager, probe, observed))
}

#[allow(clippy::too_many_arguments)]
fn record_sample(
    log: &mut File,
    path: &Path,
    payload: usize,
    keys: &[String],
    sample: usize,
    completion: &str,
    value: usize,
    burst_ns: u64,
    completion_ns: u64,
    before: &ConfigPersistenceReport,
    after: &ConfigPersistenceReport,
    probe: &Arc<Mutex<Probe>>,
    mut errors: Vec<String>,
    runtime: &CoreRuntime,
    exited: bool,
) -> Result<(), String> {
    if let Some(error) = &after.last_error {
        errors.push(format!("actual persistence error: {error}"));
    }
    let dirty = checked_delta(
        "dirty_generation",
        before.dirty_generation,
        after.dirty_generation,
        &mut errors,
    );
    let writes = checked_delta(
        "successful_writes",
        before.successful_writes,
        after.successful_writes,
        &mut errors,
    );
    let attempts = checked_delta(
        "flush_attempts",
        before.flush_attempts,
        after.flush_attempts,
        &mut errors,
    );
    let failed = checked_delta(
        "failed_writes",
        before.failed_writes,
        after.failed_writes,
        &mut errors,
    );
    let bytes = checked_delta(
        "serialized_bytes",
        before.serialized_bytes,
        after.serialized_bytes,
        &mut errors,
    );
    if dirty != keys.len() as u64
        || after.persisted_generation != after.dirty_generation
        || after.pending_flushes != 0
        || failed != 0
    {
        errors.push("dirty/persisted/pending/failure conservation".into());
    }
    if completion == "last_owner_shutdown" && !exited {
        errors.push("config worker not exited".into());
    }
    let probe = probe.lock().expect("config matrix probe");
    let attempt_bytes: u64 = probe.writes.iter().map(|w| w.serialized_bytes as u64).sum();
    if attempts != probe.writes.len() as u64
        || writes != attempts
        || bytes != attempt_bytes
        || attempts == 0
    {
        errors.push("actual attempt/write/serialized byte conservation".into());
    }
    let raw_writes: Vec<Value> = probe.writes.iter().enumerate().map(|(index,w)| {
        if w.caller_thread || w.error.is_some() || !w.commit_attempted {
            errors.push("non-background or failed real writer".into());
        }
        json!({ "attempt_index": index, "snapshot_started_ns": w.snapshot.started_ns,
            "snapshot_finished_ns": w.snapshot.finished_ns, "writer_entered_ns": w.writer_entered_ns,
            "stage_finished_ns": w.stage_finished_ns, "commit_finished_ns": w.commit_finished_ns,
            "snapshot_ns": w.snapshot.finished_ns.saturating_sub(w.snapshot.started_ns),
            "serialization_return_dispatch_ns": w.writer_entered_ns.saturating_sub(w.snapshot.finished_ns),
            "stage_write_sync_ns": w.stage_finished_ns.saturating_sub(w.writer_entered_ns),
            "fence_commit_replace_barriers_ns": w.commit_finished_ns.saturating_sub(w.stage_finished_ns),
            "serialized_bytes": w.serialized_bytes, "commit_attempted": w.commit_attempted,
            "caller_thread": w.caller_thread, "error": w.error })
    }).collect();
    drop(probe);
    let snapshot_started = Instant::now();
    let current = runtime.handle().snapshot_config_values();
    let independent_snapshot_ns = ns(snapshot_started.elapsed());
    let serialize_started = Instant::now();
    let serialized = serde_json::to_vec_pretty(&current);
    let independent_serialize_ns = ns(serialize_started.elapsed());
    let actual_serialized_bytes = match &serialized {
        Ok(bytes) => Some(bytes.len()),
        Err(error) => {
            errors.push(format!("independent serializer: {error}"));
            None
        }
    };
    drop(serialized);
    if current
        .get("runtime03.payload")
        .and_then(Value::as_str)
        .is_none_or(|s| s.len() != payload || !s.bytes().all(|b| b == b'x'))
        || current.len() != keys.len() + 1
        || keys
            .iter()
            .any(|k| current.get(k) != Some(&Value::from(value)))
    {
        errors.push("actual Core payload/key/value completeness".into());
    }
    let (digest, file_bytes) = match fs::read(path) {
        Ok(file) => {
            match serde_json::from_slice::<HashMap<String, Value>>(&file) {
                Ok(disk) if disk == current => (),
                Ok(_) => {
                    errors.push("committed file differs from actual complete Core snapshot".into())
                }
                Err(error) => errors.push(format!("committed file JSON: {error}")),
            }
            (
                Some(blake3::hash(&file).to_hex().to_string()),
                Some(file.len()),
            )
        }
        Err(error) => {
            errors.push(format!("committed file read: {error}"));
            (None, None)
        }
    };
    // No audit error above discards completed persistence timing or writer-attempt records.
    let row = json!({ "marker": "RUNTIME03_CONFIG_MATRIX_SAMPLE_V1", "payload_bytes":payload,
        "distinct_keys":keys.len(), "sample_index":sample, "completion":completion,
        "burst_ns":burst_ns, "completion_ns":completion_ns,
        "dirty_generation_delta":dirty, "persisted_generation":after.persisted_generation,
        "pending_flushes":after.pending_flushes, "flush_attempts_delta":attempts,
        "successful_writes_delta":writes, "failed_writes_delta":failed,
        "serialized_bytes_delta":bytes,
        "persistence_error_before":before.last_error, "persistence_error_after":after.last_error,
        "coalescing_dirty_per_successful_write":
            if writes>0 {Some(dirty as f64/writes as f64)} else {None},
        "report_recent_attempt_p95_ms":after.flush_p95_ms, "report_recent_attempt_max_ms":after.max_flush_ms,
        "report_recent_attempt_window_max_count":64, "report_recent_attempt_window_includes_failures":true,
        "config_worker_exited":exited, "actual_attempts":raw_writes,
        "independent_actual_snapshot_ns":independent_snapshot_ns,
        "independent_actual_serialize_ns":independent_serialize_ns,
        "independent_actual_serialized_bytes":actual_serialized_bytes,
        "committed_file_bytes":file_bytes, "committed_file_blake3":digest,
        "fixture_path":path, "complete":errors.is_empty(), "errors":errors,
        "shutdown_budget": "wait budget only; complete last-owner Drop always joins and can exceed it",
        "within_shutdown_wait_budget": completion!="last_owner_shutdown" || completion_ns<=ns(DEFAULT_SHUTDOWN_FLUSH_TIMEOUT),
        "numeric_performance_budget":null });
    emit(log, &row).map_err(|e| e.to_string())?;
    if errors.is_empty() {
        Ok(())
    } else {
        Err(format!("config shape/sample contract failed: {errors:?}"))
    }
}

fn checked_delta(name: &str, before: u64, after: u64, errors: &mut Vec<String>) -> u64 {
    match after.checked_sub(before) {
        Some(value) => value,
        None => {
            errors.push(format!("{name} regressed: {before} -> {after}"));
            0
        }
    }
}

fn checked_output() -> Result<PathBuf, String> {
    let provided =
        std::env::var("ZIRCON_RUNTIME03_CONFIG_MATRIX_DIR").map_err(|e| e.to_string())?;
    let lexical = provided.replace('/', r"\");
    if !lexical.eq_ignore_ascii_case(OUTPUT) {
        return Err("nonregistered or aliased config output path".into());
    }
    let path = PathBuf::from(OUTPUT);
    let physical = fs::canonicalize(&path).map_err(|e| e.to_string())?;
    let physical = physical.to_string_lossy();
    let physical = physical.strip_prefix(r"\\?\").unwrap_or(&physical);
    if !physical.eq_ignore_ascii_case(OUTPUT) {
        return Err("output resolves outside registered physical path".into());
    }
    let mut ancestor = Some(path.as_path());
    while let Some(p) = ancestor {
        if fs::symlink_metadata(p)
            .map_err(|e| e.to_string())?
            .file_attributes()
            & 0x400
            != 0
        {
            return Err("reparse alias in config output ancestry".into());
        }
        ancestor = p.parent().filter(|p| !p.as_os_str().is_empty());
    }
    for name in ["context.json"] {
        if fs::symlink_metadata(path.join(name))
            .map_err(|e| e.to_string())?
            .file_attributes()
            & 0x400
            != 0
        {
            return Err("aliased config context".into());
        }
    }
    Ok(path)
}

fn ns(duration: Duration) -> u64 {
    duration.as_nanos().min(u128::from(u64::MAX)) as u64
}
fn elapsed_ns(origin: Instant) -> u64 {
    ns(origin.elapsed())
}
fn emit(log: &mut File, value: &Value) -> io::Result<()> {
    serde_json::to_writer(&mut *log, value).map_err(io::Error::other)?;
    log.write_all(b"\n")?;
    log.flush()
}
