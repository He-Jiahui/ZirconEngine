use std::fs::{self, File};
use std::hint::black_box;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use super::*;
use crate::test_profile::{
    begin_allocation_profile, finish_allocation_profile, AllocationSnapshot,
};

const MIN_PROFILE_SAMPLE_COUNT: usize = 31;
const DEFAULT_PROFILE_WARMUP_COUNT: usize = 3;
const PROFILE_BATCH_SIZES: [usize; 3] = [1, 16, 256];
const PROFILE_PAYLOAD_BYTES: usize = 128;
const PROFILE_DIRECTORY_ENV: &str = "ZR_DURABLE_IO_PROFILE_DIR";
const PROFILE_SAMPLES_ENV: &str = "ZR_DURABLE_IO_PROFILE_SAMPLES";
const PROFILE_WARMUPS_ENV: &str = "ZR_DURABLE_IO_PROFILE_WARMUPS";

#[derive(Clone, Copy, Debug)]
struct ProfileSample {
    batch_size: usize,
    elapsed_ns: u64,
    allocations: AllocationSnapshot,
}

/// Measures the complete durable-commit boundary, including create-new staging, WAL publication,
/// replacement and directory synchronization. Fixture construction, validation reads and cleanup
/// remain outside the measured interval.
#[test]
#[ignore = "requires an explicit non-C profile directory and a release single-thread run"]
fn durable_transaction_commit_current_source_profile() {
    assert!(
        !cfg!(debug_assertions),
        "durable I/O profile must run with cargo test --release"
    );
    let report_directory = profile_report_directory();
    let sample_count =
        profile_count(PROFILE_SAMPLES_ENV, MIN_PROFILE_SAMPLE_COUNT).max(MIN_PROFILE_SAMPLE_COUNT);
    let warmup_count = profile_count(PROFILE_WARMUPS_ENV, DEFAULT_PROFILE_WARMUP_COUNT);
    let work_root = report_directory.join(format!(
        "durable-io-profile-work-{}-{}",
        std::process::id(),
        crate::io::next_test_output_id()
    ));
    fs::create_dir_all(&work_root).expect("create durable I/O profile work root");

    let mut samples = Vec::with_capacity(PROFILE_BATCH_SIZES.len() * sample_count);
    for batch_size in PROFILE_BATCH_SIZES {
        for warmup in 0..warmup_count {
            let _ = execute_sample(&work_root, batch_size, &format!("warmup-{warmup}"), false);
        }
        for sample in 0..sample_count {
            samples.push(execute_sample(
                &work_root,
                batch_size,
                &format!("sample-{sample}"),
                true,
            ));
        }
    }

    write_profile_reports(&report_directory, sample_count, warmup_count, &samples);
    fs::remove_dir_all(work_root).expect("remove durable I/O profile work root");
    println!(
        "ZR_DURABLE_IO_PROFILE_REPORT path={}",
        report_directory.display()
    );
}

fn execute_sample(
    work_root: &Path,
    batch_size: usize,
    sample_label: &str,
    measured: bool,
) -> ProfileSample {
    let sample_root = work_root.join(format!("batch-{batch_size}-{sample_label}"));
    let journal_directory = sample_root.join("journal");
    fs::create_dir_all(&sample_root).expect("create durable I/O profile sample root");
    let writes = (0..batch_size)
        .map(|index| {
            let payload_byte = u8::try_from(index % 251).expect("profile payload byte is bounded");
            PreparedFileWrite::new(
                sample_root.join(format!("asset-{index:03}.bin")),
                vec![payload_byte; PROFILE_PAYLOAD_BYTES],
            )
        })
        .collect::<Vec<_>>();
    let first_target = writes
        .first()
        .expect("profile batch is non-empty")
        .path
        .clone();
    let last_target = writes
        .last()
        .expect("profile batch is non-empty")
        .path
        .clone();
    let mut report = DurableCommitReport::default();

    if measured {
        begin_allocation_profile();
    }
    let started = Instant::now();
    let disposition = commit_prepared_files(
        &journal_directory,
        "profile",
        writes,
        TransactionFault::None,
        &mut report,
    )
    .expect("profile transaction must commit");
    let elapsed = started.elapsed();
    let allocations = if measured {
        finish_allocation_profile()
    } else {
        AllocationSnapshot::default()
    };

    assert_eq!(disposition, DurableCommitDisposition::Durable);
    assert_eq!(
        black_box(
            fs::metadata(first_target)
                .expect("first profile target exists")
                .len()
        ),
        PROFILE_PAYLOAD_BYTES as u64
    );
    assert_eq!(
        black_box(
            fs::metadata(last_target)
                .expect("last profile target exists")
                .len()
        ),
        PROFILE_PAYLOAD_BYTES as u64
    );
    assert_eq!(
        fs::read_dir(&journal_directory)
            .expect("profile journal directory exists")
            .count(),
        0,
        "durable commit must leave no journal evidence"
    );
    fs::remove_dir_all(sample_root).expect("remove durable I/O profile sample root");

    ProfileSample {
        batch_size,
        elapsed_ns: duration_ns(elapsed),
        allocations,
    }
}

fn profile_count(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .map(|value| value.parse::<usize>().expect("profile count must be usize"))
        .unwrap_or(default)
}

fn profile_report_directory() -> PathBuf {
    let directory = PathBuf::from(
        std::env::var_os(PROFILE_DIRECTORY_ENV).expect("ZR_DURABLE_IO_PROFILE_DIR must be set"),
    );
    assert!(
        directory.is_absolute(),
        "durable I/O profile directory must be absolute"
    );
    assert_profile_directory_is_not_on_c_drive(&directory);
    fs::create_dir_all(&directory).expect("create durable I/O profile report directory");
    directory
}

#[cfg(windows)]
fn assert_profile_directory_is_not_on_c_drive(directory: &Path) {
    use std::path::{Component, Prefix};

    let prefix = directory
        .components()
        .next()
        .and_then(|component| match component {
            Component::Prefix(prefix) => Some(prefix.kind()),
            _ => None,
        })
        .expect("Windows profile directory must have a drive or UNC prefix");
    match prefix {
        Prefix::Disk(letter) | Prefix::VerbatimDisk(letter) => {
            assert_ne!(letter.to_ascii_uppercase(), b'C', "C drive is forbidden")
        }
        _ => {}
    }
}

#[cfg(not(windows))]
fn assert_profile_directory_is_not_on_c_drive(_directory: &Path) {}

fn duration_ns(duration: Duration) -> u64 {
    u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
}

fn percentile(values: &[u64], percentile: usize) -> u64 {
    assert!(!values.is_empty());
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).saturating_add(99) / 100;
    sorted[rank.saturating_sub(1).min(sorted.len() - 1)]
}

fn median_absolute_deviation(values: &[u64], median: u64) -> u64 {
    percentile(
        &values
            .iter()
            .map(|value| value.abs_diff(median))
            .collect::<Vec<_>>(),
        50,
    )
}

fn source_blake3(source: &str) -> String {
    blake3::hash(source.as_bytes()).to_hex().to_string()
}

fn write_profile_reports(
    directory: &Path,
    sample_count: usize,
    warmup_count: usize,
    samples: &[ProfileSample],
) {
    let artifact_identity_source = source_blake3(include_str!("../../../artifact_identity.rs"));
    let transaction_pathing_source = source_blake3(include_str!("../../pathing.rs"));
    let transaction_engine_source = source_blake3(include_str!("../../engine.rs"));
    let journal_schema_source = source_blake3(include_str!("../../schema.rs"));
    let raw_path = directory.join("durable-io-current-raw-samples.csv");
    let summary_path = directory.join("durable-io-current-summary.csv");
    let metadata_path = directory.join("durable-io-current-metadata.txt");

    let mut raw = BufWriter::new(File::create(raw_path).expect("create durable I/O raw CSV"));
    writeln!(
        raw,
        "batch_size,payload_bytes_per_write,sample,elapsed_ns,allocation_count,requested_bytes,peak_live_bytes"
    )
    .expect("write durable I/O raw header");
    for (index, sample) in samples.iter().enumerate() {
        writeln!(
            raw,
            "{},{},{},{},{},{},{}",
            sample.batch_size,
            PROFILE_PAYLOAD_BYTES,
            index % sample_count,
            sample.elapsed_ns,
            sample.allocations.allocation_count,
            sample.allocations.requested_bytes,
            sample.allocations.peak_live_bytes
        )
        .expect("write durable I/O raw row");
    }
    raw.flush().expect("flush durable I/O raw CSV");

    let mut summary =
        BufWriter::new(File::create(summary_path).expect("create durable I/O summary CSV"));
    writeln!(
        summary,
        "batch_size,payload_bytes_per_write,samples,warmups,p50_ns,p95_ns,mad_ns,allocation_count_p50,requested_bytes_p50,peak_live_bytes_p50,artifact_identity_source_blake3,transaction_pathing_source_blake3,transaction_engine_source_blake3,journal_schema_source_blake3"
    )
    .expect("write durable I/O summary header");
    for batch_size in PROFILE_BATCH_SIZES {
        let batch_samples = samples
            .iter()
            .copied()
            .filter(|sample| sample.batch_size == batch_size)
            .collect::<Vec<_>>();
        let elapsed = batch_samples
            .iter()
            .map(|sample| sample.elapsed_ns)
            .collect::<Vec<_>>();
        let allocation_count = batch_samples
            .iter()
            .map(|sample| sample.allocations.allocation_count)
            .collect::<Vec<_>>();
        let requested_bytes = batch_samples
            .iter()
            .map(|sample| sample.allocations.requested_bytes)
            .collect::<Vec<_>>();
        let peak_live_bytes = batch_samples
            .iter()
            .map(|sample| sample.allocations.peak_live_bytes)
            .collect::<Vec<_>>();
        let p50 = percentile(&elapsed, 50);
        writeln!(
            summary,
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            batch_size,
            PROFILE_PAYLOAD_BYTES,
            batch_samples.len(),
            warmup_count,
            p50,
            percentile(&elapsed, 95),
            median_absolute_deviation(&elapsed, p50),
            percentile(&allocation_count, 50),
            percentile(&requested_bytes, 50),
            percentile(&peak_live_bytes, 50),
            artifact_identity_source,
            transaction_pathing_source,
            transaction_engine_source,
            journal_schema_source
        )
        .expect("write durable I/O summary row");
    }
    summary.flush().expect("flush durable I/O summary CSV");

    fs::write(
        metadata_path,
        format!(
            "schema=zr_durable_io_profile_v1\nprofile=release\nrequired_test_threads=1\nsamples={sample_count}\nwarmups={warmup_count}\npayload_bytes_per_write={PROFILE_PAYLOAD_BYTES}\nartifact_identity_source_blake3={artifact_identity_source}\ntransaction_pathing_source_blake3={transaction_pathing_source}\ntransaction_engine_source_blake3={transaction_engine_source}\njournal_schema_source_blake3={journal_schema_source}\nrss=unavailable\nio_counters=unavailable\npower=unavailable\n"
        ),
    )
    .expect("write durable I/O profile metadata");
}
