use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use super::RollingFileLogSink;
use crate::core::logging::{LogEntry, LogRecord, LogSeverity, LogSource};

static NEXT_TEMP_DIRECTORY: AtomicU64 = AtomicU64::new(0);

fn temp_directory(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "zircon_editor_rolling_{label}_{}_{}",
        std::process::id(),
        NEXT_TEMP_DIRECTORY.fetch_add(1, Ordering::Relaxed),
    ))
}

fn record(sequence: u64) -> LogRecord {
    LogRecord::new(
        sequence,
        LogEntry::new(
            LogSource::editor(),
            LogSeverity::Info,
            "cached rolling segment",
            sequence,
            None,
        )
        .unwrap(),
    )
}

#[test]
fn optimization_wave_20260824c_editor11_stable_segment_reuses_file_control_path() {
    const WRITES: u64 = 64;

    let directory = temp_directory("reuse");
    let sink = RollingFileLogSink::new(&directory, 1 << 20).unwrap();
    for sequence in 1..=WRITES {
        sink.append_for_day(&record(sequence), 20_000).unwrap();
    }

    let counters = sink.io_counters();
    assert_eq!(counters.directory_preparations, 1);
    assert_eq!(counters.metadata_probes, 1);
    assert_eq!(counters.file_opens, 1);
    assert_eq!(counters.flushes, WRITES);

    drop(sink);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn optimization_wave_20260824c_editor11_cached_segment_rotates_when_full() {
    let directory = temp_directory("rotation");
    let sink = RollingFileLogSink::new(&directory, 1).unwrap();

    let first = sink.append_for_day(&record(1), 20_000).unwrap();
    let second = sink.append_for_day(&record(2), 20_000).unwrap();

    assert_ne!(first, second);
    assert!(first
        .file_name()
        .unwrap()
        .to_string_lossy()
        .contains("-0.log"));
    assert!(second
        .file_name()
        .unwrap()
        .to_string_lossy()
        .contains("-1.log"));
    let counters = sink.io_counters();
    assert_eq!(counters.file_opens, 2);
    assert_eq!(counters.flushes, 2);

    drop(sink);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
#[ignore = "managed release performance evidence"]
fn optimization_wave_20260824c_editor11_rolling_segment_cache_evidence() {
    const WRITES: u64 = 2_000;
    const MAX_ELAPSED_NS: u128 = 5_000_000_000;

    let directory = temp_directory("evidence");
    let sink = RollingFileLogSink::new(&directory, 1 << 20).unwrap();
    let started = Instant::now();
    for sequence in 1..=WRITES {
        sink.append_for_day(&record(sequence), 20_000).unwrap();
    }
    let elapsed_ns = started.elapsed().as_nanos();
    let counters = sink.io_counters();
    let legacy_control_operations = WRITES.saturating_mul(3);
    let optimized_control_operations = counters
        .directory_preparations
        .saturating_add(counters.metadata_probes)
        .saturating_add(counters.file_opens);
    let control_operation_reduction_bps = legacy_control_operations
        .saturating_sub(optimized_control_operations)
        .saturating_mul(10_000)
        / legacy_control_operations;

    println!(
        "EDITOR_ROLLING_LOG_BENCH_V1 writes={WRITES} legacy_control_operations={legacy_control_operations} optimized_control_operations={optimized_control_operations} control_operation_reduction_bps={control_operation_reduction_bps} flushes={} elapsed_ns={elapsed_ns} max_elapsed_ns={MAX_ELAPSED_NS}",
        counters.flushes,
    );

    assert_eq!(optimized_control_operations, 3);
    assert!(control_operation_reduction_bps >= 9_990);
    assert_eq!(counters.flushes, WRITES);
    assert!(elapsed_ns <= MAX_ELAPSED_NS);

    drop(sink);
    std::fs::remove_dir_all(directory).unwrap();
}
