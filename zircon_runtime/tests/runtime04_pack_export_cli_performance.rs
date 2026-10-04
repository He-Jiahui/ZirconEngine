//! Product-path comparison for two externally built, source-matched pack exporters.
//! The harness prepares inputs outside the timed region and invokes the real CLI.

#![cfg(windows)]

use std::ffi::{c_void, OsStr};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::os::windows::io::AsRawHandle;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

const WARMUP_PAIRS: usize = 5;
const SAMPLE_PAIRS: usize = 31;
const POLL_INTERVAL: Duration = Duration::from_millis(1);

#[test]
#[ignore = "Windows Release product baseline; prints raw CLI timing and memory samples"]
fn runtime04_pack_export_cli_current_1k() {
    benchmark_current_cli(Workload::Pack {
        assets: 1_000,
        source_bytes: 16 * 1_024,
    });
}

#[test]
#[ignore = "Windows Release product baseline; prints raw CLI timing and memory samples"]
fn runtime04_pack_export_cli_current_100k() {
    benchmark_current_cli(Workload::Pack {
        assets: 100_000,
        source_bytes: 256,
    });
}

#[test]
#[ignore = "requires two managed Windows Release binaries from matched source snapshots"]
fn runtime04_pack_export_cli_product_1k() {
    compare_cli(Workload::Pack {
        assets: 1_000,
        source_bytes: 16 * 1_024,
    });
}

#[test]
#[ignore = "requires two managed Windows Release binaries from matched source snapshots"]
fn runtime04_pack_export_cli_product_100k() {
    compare_cli(Workload::Pack {
        assets: 100_000,
        source_bytes: 256,
    });
}

#[test]
#[ignore = "requires two managed Windows Release binaries from matched source snapshots"]
fn runtime04_pack_export_cli_delta_product_1k() {
    compare_cli(Workload::Delta);
}

#[derive(Clone, Copy)]
enum Workload {
    Pack { assets: usize, source_bytes: usize },
    Delta,
}

impl Workload {
    fn label(self) -> &'static str {
        match self {
            Self::Pack { assets: 1_000, .. } => "pack_1k",
            Self::Pack {
                assets: 100_000, ..
            } => "pack_100k",
            Self::Pack { .. } => "pack_other",
            Self::Delta => "delta_1k",
        }
    }

    fn child_timeout(self) -> Duration {
        match self {
            Self::Pack {
                assets: 100_000, ..
            } => Duration::from_secs(300),
            Self::Pack { .. } => Duration::from_secs(90),
            Self::Delta => Duration::from_secs(180),
        }
    }

    fn suite_timeout(self, compare: bool) -> Duration {
        match (self, compare) {
            (
                Self::Pack {
                    assets: 100_000, ..
                },
                false,
            ) => Duration::from_secs(1_800),
            (
                Self::Pack {
                    assets: 100_000, ..
                },
                true,
            ) => Duration::from_secs(3_600),
            (_, false) => Duration::from_secs(600),
            (_, true) => Duration::from_secs(1_200),
        }
    }
}

struct Binaries {
    baseline: PathBuf,
    candidate: PathBuf,
    matched_source_id: String,
    baseline_hash: String,
    candidate_hash: String,
}

impl Binaries {
    fn from_environment() -> Self {
        assert!(!cfg!(debug_assertions), "run the CLI comparison in Release");
        let baseline = required_path("ZR_RUNTIME04_PACK_BASELINE_EXE");
        let candidate = required_path("ZR_RUNTIME04_PACK_CANDIDATE_EXE");
        let matched_source_id = std::env::var("ZR_RUNTIME04_PACK_MATCHED_SOURCE_ID")
            .expect("set the matched source fingerprint for both binaries");
        assert!(!matched_source_id.trim().is_empty());
        assert!(baseline.is_file() && candidate.is_file());
        assert_ne!(
            baseline.canonicalize().unwrap(),
            candidate.canonicalize().unwrap(),
            "baseline and candidate must be distinct executable paths"
        );
        let baseline_hash = file_hash(&baseline);
        let candidate_hash = file_hash(&candidate);
        assert_ne!(baseline_hash, candidate_hash, "binaries must differ");
        Self {
            baseline,
            candidate,
            matched_source_id,
            baseline_hash,
            candidate_hash,
        }
    }
}

fn required_path(name: &str) -> PathBuf {
    PathBuf::from(std::env::var_os(name).unwrap_or_else(|| panic!("set {name}")))
}

#[derive(Serialize)]
struct Manifest {
    roots: Vec<String>,
    assets: Vec<ManifestAsset>,
}

#[derive(Serialize)]
struct ManifestAsset {
    path: String,
    source: String,
}

struct Corpus {
    root: PathBuf,
    workload: Workload,
    target_manifest: PathBuf,
    base_pack: Option<PathBuf>,
    expected_assets: usize,
    expected_chunks: usize,
    expected_delta_changed: usize,
    expected_delta_removed: usize,
    expected_delta_reused: usize,
}

impl Corpus {
    fn prepare(workload: Workload, candidate_exe: &Path) -> Self {
        let root = unique_temp_root(workload.label());
        let mut cleanup_on_failure = TempRootCleanup(Some(root.clone()));
        fs::create_dir_all(root.join("sources")).unwrap();
        fs::create_dir_all(root.join("baseline_output")).unwrap();
        fs::create_dir_all(root.join("candidate_output")).unwrap();
        match workload {
            Workload::Pack {
                assets,
                source_bytes,
            } => {
                let unique = assets * 9 / 10;
                let mut entries = Vec::with_capacity(assets);
                for index in 0..assets {
                    let source_index = index % unique;
                    let source = format!("sources/pack_{source_index:06}.bin");
                    if index < unique {
                        write_source(&root.join(&source), source_bytes, source_index, 0);
                    }
                    entries.push(ManifestAsset {
                        path: format!("assets/{index:06}.bin"),
                        source,
                    });
                }
                let target_manifest = root.join("target.json");
                write_manifest(&target_manifest, entries);
                cleanup_on_failure.0 = None;
                Self {
                    root,
                    workload,
                    target_manifest,
                    base_pack: None,
                    expected_assets: assets,
                    expected_chunks: unique,
                    expected_delta_changed: 0,
                    expected_delta_removed: 0,
                    expected_delta_reused: 0,
                }
            }
            Workload::Delta => {
                let mut base = Vec::with_capacity(1_000);
                let mut target = Vec::with_capacity(1_000);
                for index in 0..1_000 {
                    let source = format!("sources/base_{index:06}.bin");
                    write_source(&root.join(&source), 16 * 1_024, index, 0);
                    base.push(ManifestAsset {
                        path: format!("assets/{index:06}.bin"),
                        source: source.clone(),
                    });
                    if (250..750).contains(&index) {
                        let changed = format!("sources/changed_{index:06}.bin");
                        write_source(&root.join(&changed), 16 * 1_024, index, 1);
                        target.push(ManifestAsset {
                            path: format!("assets/{index:06}.bin"),
                            source: changed,
                        });
                    } else if index >= 750 {
                        target.push(ManifestAsset {
                            path: format!("assets/{index:06}.bin"),
                            source,
                        });
                    }
                }
                for index in 1_000..1_250 {
                    let source = format!("sources/new_{index:06}.bin");
                    write_source(&root.join(&source), 16 * 1_024, index, 2);
                    target.push(ManifestAsset {
                        path: format!("assets/{index:06}.bin"),
                        source,
                    });
                }
                let base_manifest = root.join("base.json");
                let target_manifest = root.join("target.json");
                write_manifest(&base_manifest, base);
                write_manifest(&target_manifest, target);
                let base_pack = root.join("base.zrpack");
                let base_report = root.join("base-report.json");
                let base_stderr = root.join("base-stderr.txt");
                let stderr_file = File::create(&base_stderr).expect("create base-pack stderr file");
                let mut child = Command::new(candidate_exe)
                    .args(cli_args(&base_manifest, &base_pack, &base_report, None))
                    .stdout(Stdio::null())
                    .stderr(Stdio::from(stderr_file))
                    .spawn()
                    .expect("launch base-pack preparation with real CLI");
                let started = Instant::now();
                let status = loop {
                    if let Some(status) = child.try_wait().expect("wait for base-pack preparation")
                    {
                        break status;
                    }
                    if started.elapsed() >= workload.child_timeout() {
                        abort_timed_out_child(
                            &mut child,
                            &base_stderr,
                            "base-pack preparation",
                            workload.child_timeout(),
                        );
                    }
                    thread::sleep(POLL_INTERVAL);
                };
                assert!(
                    status.success(),
                    "base-pack preparation failed with {status}; stderr: {}",
                    fs::read_to_string(&base_stderr)
                        .unwrap_or_else(|error| format!("unreadable: {error}"))
                );
                let summary = read_report(&base_report);
                assert!(!summary.fatal && summary.asset_count == 1_000);
                cleanup_on_failure.0 = None;
                Self {
                    root,
                    workload,
                    target_manifest,
                    base_pack: Some(base_pack),
                    expected_assets: 1_000,
                    expected_chunks: 1_000,
                    expected_delta_changed: 750,
                    expected_delta_removed: 250,
                    expected_delta_reused: 250,
                }
            }
        }
    }

    fn validate(&self, result: &RunResult) {
        let report = &result.report;
        assert!(!report.fatal);
        assert_eq!(report.asset_count, self.expected_assets);
        assert_eq!(report.chunk_count, self.expected_chunks);
        assert_eq!(
            report.deduplicated_assets.len(),
            self.expected_assets - self.expected_chunks
        );
        assert_eq!(report.delta_asset_count, self.expected_delta_changed);
        assert_eq!(
            report.delta_removed_assets.len(),
            self.expected_delta_removed
        );
        assert_eq!(report.delta_reused_assets.len(), self.expected_delta_reused);
        assert_eq!(report.delta_apply_verified, self.base_pack.is_some());
        assert_eq!(result.delta_hash.is_some(), self.base_pack.is_some());
    }
}

impl Drop for Corpus {
    fn drop(&mut self) {
        cleanup_temp_root(&self.root);
    }
}

struct TempRootCleanup(Option<PathBuf>);

impl Drop for TempRootCleanup {
    fn drop(&mut self) {
        if let Some(root) = self.0.as_deref() {
            cleanup_temp_root(root);
        }
    }
}

fn cleanup_temp_root(path: &Path) {
    let Ok(root) = path.canonicalize() else {
        return;
    };
    let Ok(temp) = std::env::temp_dir().canonicalize() else {
        return;
    };
    let named_for_fixture = root
        .file_name()
        .and_then(OsStr::to_str)
        .is_some_and(|name| name.starts_with("zircon-runtime04-pack-cli-"));
    if root.parent() == Some(temp.as_path()) && named_for_fixture {
        let _ = fs::remove_dir_all(root);
    }
}

fn write_source(path: &Path, bytes: usize, index: usize, variant: u64) {
    let mut payload = vec![0_u8; bytes];
    payload[..8].copy_from_slice(&(index as u64).to_le_bytes());
    payload[8..16].copy_from_slice(&variant.to_le_bytes());
    for (offset, byte) in payload[16..].iter_mut().enumerate() {
        *byte = (index as u8)
            .wrapping_add(offset as u8)
            .wrapping_add(variant as u8);
    }
    fs::write(path, payload).expect("write deterministic source");
}

fn write_manifest(path: &Path, assets: Vec<ManifestAsset>) {
    let roots = assets.iter().map(|asset| asset.path.clone()).collect();
    let manifest = Manifest { roots, assets };
    let file = File::create(path).expect("create pack manifest");
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, &manifest).expect("serialize pack manifest");
    writer.flush().expect("flush pack manifest");
}

fn unique_temp_root(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "zircon-runtime04-pack-cli-{label}-{}-{nanos}",
        std::process::id()
    ))
}

#[derive(Deserialize)]
struct PackReport {
    fatal: bool,
    asset_count: usize,
    chunk_count: usize,
    deduplicated_assets: Vec<String>,
    delta_asset_count: usize,
    delta_removed_assets: Vec<String>,
    delta_reused_assets: Vec<String>,
    delta_apply_verified: bool,
}

struct RunResult {
    wall_ns: u128,
    peak_working_set_bytes: u64,
    peak_commit_bytes: u64,
    complete_memory_peak: bool,
    pack_hash: String,
    delta_hash: Option<String>,
    report: PackReport,
}

fn read_report(path: &Path) -> PackReport {
    serde_json::from_reader(BufReader::new(File::open(path).expect("open CLI report")))
        .expect("decode CLI report")
}

fn cli_args(
    manifest: &Path,
    pack: &Path,
    report: &Path,
    delta: Option<(&Path, &Path)>,
) -> Vec<std::ffi::OsString> {
    let mut args = vec![
        "--profile".into(),
        "windows-release".into(),
        "--manifest".into(),
        manifest.as_os_str().to_owned(),
        "--pack".into(),
        pack.as_os_str().to_owned(),
        "--report".into(),
        report.as_os_str().to_owned(),
    ];
    if let Some((base, delta_pack)) = delta {
        args.extend([
            "--previous-pack".into(),
            base.as_os_str().to_owned(),
            "--delta-pack".into(),
            delta_pack.as_os_str().to_owned(),
        ]);
    }
    args
}

fn run_cli(binary: &Path, corpus: &Corpus, output: &Path) -> RunResult {
    let pack = output.join("out.zrpack");
    let report = output.join("report.json");
    let delta = output.join("out.zrpd");
    let stderr = output.join("stderr.txt");
    for path in [&pack, &report, &delta, &stderr] {
        if path.exists() {
            fs::remove_file(path).expect("remove prior sample output");
        }
    }
    let delta_input = corpus
        .base_pack
        .as_deref()
        .map(|base| (base, delta.as_path()));
    let stderr_file = File::create(&stderr).expect("create CLI stderr file");
    let started = Instant::now();
    let mut child = Command::new(binary)
        .args(cli_args(
            &corpus.target_manifest,
            &pack,
            &report,
            delta_input,
        ))
        .stdout(Stdio::null())
        .stderr(Stdio::from(stderr_file))
        .spawn()
        .expect("launch real pack exporter");
    let mut observed_working_set = 0_u64;
    let mut observed_commit = 0_u64;
    let status = loop {
        if let Some(memory) = process_memory(&child) {
            observed_working_set = observed_working_set.max(memory.peak_working_set_size as u64);
            observed_commit = observed_commit.max(memory.peak_pagefile_usage as u64);
        }
        if let Some(status) = child.try_wait().expect("wait for pack exporter") {
            break status;
        }
        if started.elapsed() >= corpus.workload.child_timeout() {
            abort_timed_out_child(
                &mut child,
                &stderr,
                corpus.workload.label(),
                corpus.workload.child_timeout(),
            );
        }
        thread::sleep(POLL_INTERVAL);
    };
    let wall_ns = started.elapsed().as_nanos();
    let final_memory = process_memory(&child);
    let complete_memory_peak = final_memory.as_ref().is_some_and(|memory| {
        memory.peak_working_set_size > 0
            && memory.peak_pagefile_usage > 0
            && memory.peak_working_set_size as u64 >= observed_working_set
            && memory.peak_pagefile_usage as u64 >= observed_commit
    });
    if let Some(memory) = final_memory.as_ref() {
        observed_working_set = observed_working_set.max(memory.peak_working_set_size as u64);
        observed_commit = observed_commit.max(memory.peak_pagefile_usage as u64);
    }
    assert!(
        status.success(),
        "pack exporter failed with {status}; stderr: {}",
        fs::read_to_string(&stderr).unwrap_or_else(|error| format!("unreadable: {error}"))
    );
    assert!(observed_working_set > 0, "process memory was not sampled");
    let result = RunResult {
        wall_ns,
        peak_working_set_bytes: observed_working_set,
        peak_commit_bytes: observed_commit,
        complete_memory_peak,
        pack_hash: file_hash(&pack),
        delta_hash: delta_input.map(|_| file_hash(&delta)),
        report: read_report(&report),
    };
    corpus.validate(&result);
    result
}

fn abort_timed_out_child(
    child: &mut Child,
    stderr: &Path,
    operation: &str,
    timeout: Duration,
) -> ! {
    let kill_error = child.kill().err();
    let reap_deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if child
            .try_wait()
            .expect("check timed-out child termination")
            .is_some()
        {
            break;
        }
        if kill_error.is_some() || Instant::now() >= reap_deadline {
            panic!(
                "{operation} exceeded {timeout:?}; kill_error={kill_error:?}; process did not terminate within the bounded reap window; stderr: {}",
                fs::read_to_string(stderr).unwrap_or_else(|error| format!("unreadable: {error}"))
            );
        }
        thread::sleep(POLL_INTERVAL);
    }
    panic!(
        "{operation} exceeded {timeout:?}; kill_error={kill_error:?}; stderr: {}",
        fs::read_to_string(stderr).unwrap_or_else(|error| format!("unreadable: {error}"))
    );
}

fn file_hash(path: &Path) -> String {
    let mut file = BufReader::new(File::open(path).expect("open file for digest"));
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer).expect("hash file");
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    hasher.finalize().to_hex().to_string()
}

fn compare_cli(workload: Workload) {
    let binaries = Binaries::from_environment();
    let corpus = Corpus::prepare(workload, &binaries.candidate);
    let manifest_hash = file_hash(&corpus.target_manifest);
    let baseline_output = corpus.root.join("baseline_output");
    let candidate_output = corpus.root.join("candidate_output");
    let mut baseline_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut expected_pack_hash = None;
    let mut expected_delta_hash = None;
    let suite_started = Instant::now();
    for pair in 0..(WARMUP_PAIRS + SAMPLE_PAIRS) {
        assert!(
            suite_started.elapsed() < workload.suite_timeout(true),
            "{} comparison suite exceeded {:?}",
            workload.label(),
            workload.suite_timeout(true)
        );
        let (baseline, candidate) = if pair % 2 == 0 {
            (
                run_cli(&binaries.baseline, &corpus, &baseline_output),
                run_cli(&binaries.candidate, &corpus, &candidate_output),
            )
        } else {
            let candidate = run_cli(&binaries.candidate, &corpus, &candidate_output);
            let baseline = run_cli(&binaries.baseline, &corpus, &baseline_output);
            (baseline, candidate)
        };
        assert_eq!(baseline.pack_hash, candidate.pack_hash);
        assert_eq!(baseline.delta_hash, candidate.delta_hash);
        if let Some(expected) = &expected_pack_hash {
            assert_eq!(
                &baseline.pack_hash, expected,
                "pack output changed between pairs"
            );
            assert_eq!(
                &baseline.delta_hash, &expected_delta_hash,
                "delta output changed between pairs"
            );
        } else {
            expected_pack_hash = Some(baseline.pack_hash.clone());
            expected_delta_hash = baseline.delta_hash.clone();
        }
        if pair >= WARMUP_PAIRS {
            baseline_samples.push(baseline);
            candidate_samples.push(candidate);
        }
    }
    print_and_gate(
        workload,
        &binaries,
        &corpus,
        &manifest_hash,
        &baseline_samples,
        &candidate_samples,
    );
}

fn benchmark_current_cli(workload: Workload) {
    assert!(
        !cfg!(debug_assertions),
        "run the CLI product baseline in Release"
    );
    let binary = Path::new(env!("CARGO_BIN_EXE_zircon_export_pack"));
    let binary_hash = file_hash(binary);
    let corpus = Corpus::prepare(workload, binary);
    let manifest_hash = file_hash(&corpus.target_manifest);
    let output = corpus.root.join("candidate_output");
    let mut samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut expected_pack_hash = None;
    let suite_started = Instant::now();
    for index in 0..(WARMUP_PAIRS + SAMPLE_PAIRS) {
        assert!(
            suite_started.elapsed() < workload.suite_timeout(false),
            "{} current-binary suite exceeded {:?}",
            workload.label(),
            workload.suite_timeout(false)
        );
        let sample = run_cli(binary, &corpus, &output);
        if let Some(expected) = &expected_pack_hash {
            assert_eq!(
                &sample.pack_hash, expected,
                "pack output changed between samples"
            );
        } else {
            expected_pack_hash = Some(sample.pack_hash.clone());
        }
        if index >= WARMUP_PAIRS {
            samples.push(sample);
        }
    }
    print_current_baseline(workload, &binary_hash, &manifest_hash, &corpus, &samples);
}

fn print_current_baseline(
    workload: Workload,
    binary_hash: &str,
    manifest_hash: &str,
    corpus: &Corpus,
    samples: &[RunResult],
) {
    let wall = samples
        .iter()
        .map(|sample| sample.wall_ns)
        .collect::<Vec<_>>();
    let rss = samples
        .iter()
        .map(|sample| sample.peak_working_set_bytes)
        .collect::<Vec<_>>();
    let commit = samples
        .iter()
        .map(|sample| sample.peak_commit_bytes)
        .collect::<Vec<_>>();
    let (wall_p50, wall_p95, wall_p99) = percentiles(&wall);
    let (rss_p50, rss_p95, rss_p99) = percentiles(&rss);
    let complete_memory_peak = samples.iter().all(|sample| sample.complete_memory_peak);
    println!(
        "PERF_RESULT RUNTIME04_PACK_EXPORT_CLI_CURRENT_V1 workload={} binary_hash={binary_hash} manifest_hash={manifest_hash} warmups={WARMUP_PAIRS} samples={SAMPLE_PAIRS} stdout=null poll_interval_ms=1 cache_state=warm_os_cache_uncontrolled wall_p50_ns={wall_p50} wall_p95_ns={wall_p95} wall_p99_ns={wall_p99} rss_p50_bytes={rss_p50} rss_p95_bytes={rss_p95} rss_p99_bytes={rss_p99} complete_memory_peak={complete_memory_peak} wall_raw_ns={wall:?} rss_raw_bytes={rss:?} commit_raw_bytes={commit:?} pack_hash={} os={} arch={} package_version={} logical_workers={} cpu_id={:?} storage_root={:?}",
        workload.label(),
        samples[0].pack_hash.as_str(),
        std::env::consts::OS,
        std::env::consts::ARCH,
        env!("CARGO_PKG_VERSION"),
        std::thread::available_parallelism().map_or(0, |threads| threads.get()),
        std::env::var("PROCESSOR_IDENTIFIER").ok(),
        &corpus.root,
    );
    assert!(
        complete_memory_peak,
        "final process memory counters unavailable; RSS baseline remains open"
    );
}

fn percentiles<T: Ord + Copy>(samples: &[T]) -> (T, T, T) {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = |percent: usize| sorted[(sorted.len() * percent).div_ceil(100) - 1];
    (rank(50), rank(95), rank(99))
}

fn print_and_gate(
    workload: Workload,
    binaries: &Binaries,
    corpus: &Corpus,
    manifest_hash: &str,
    baseline: &[RunResult],
    candidate: &[RunResult],
) {
    let baseline_wall = baseline
        .iter()
        .map(|sample| sample.wall_ns)
        .collect::<Vec<_>>();
    let candidate_wall = candidate
        .iter()
        .map(|sample| sample.wall_ns)
        .collect::<Vec<_>>();
    let baseline_rss = baseline
        .iter()
        .map(|sample| sample.peak_working_set_bytes)
        .collect::<Vec<_>>();
    let candidate_rss = candidate
        .iter()
        .map(|sample| sample.peak_working_set_bytes)
        .collect::<Vec<_>>();
    let baseline_commit = baseline
        .iter()
        .map(|sample| sample.peak_commit_bytes)
        .collect::<Vec<_>>();
    let candidate_commit = candidate
        .iter()
        .map(|sample| sample.peak_commit_bytes)
        .collect::<Vec<_>>();
    let (baseline_p50, baseline_p95, baseline_p99) = percentiles(&baseline_wall);
    let (candidate_p50, candidate_p95, candidate_p99) = percentiles(&candidate_wall);
    let (baseline_rss_p50, baseline_rss_p95, baseline_rss_p99) = percentiles(&baseline_rss);
    let (candidate_rss_p50, candidate_rss_p95, candidate_rss_p99) = percentiles(&candidate_rss);
    let complete_memory_peak = baseline
        .iter()
        .chain(candidate)
        .all(|sample| sample.complete_memory_peak);
    println!(
        "PERF_RESULT RUNTIME04_PACK_EXPORT_CLI_PRODUCT_V1 workload={} matched_source_id={} baseline_exe_hash={} candidate_exe_hash={} manifest_hash={manifest_hash} warmup_pairs={WARMUP_PAIRS} sample_pairs={SAMPLE_PAIRS} order=baseline_first_even_pair stdout=null poll_interval_ms=1 cache_state=warm_os_cache_uncontrolled baseline_wall_p50_ns={baseline_p50} baseline_wall_p95_ns={baseline_p95} baseline_wall_p99_ns={baseline_p99} candidate_wall_p50_ns={candidate_p50} candidate_wall_p95_ns={candidate_p95} candidate_wall_p99_ns={candidate_p99} baseline_rss_p50_bytes={baseline_rss_p50} baseline_rss_p95_bytes={baseline_rss_p95} baseline_rss_p99_bytes={baseline_rss_p99} candidate_rss_p50_bytes={candidate_rss_p50} candidate_rss_p95_bytes={candidate_rss_p95} candidate_rss_p99_bytes={candidate_rss_p99} complete_memory_peak={complete_memory_peak} baseline_wall_raw_ns={baseline_wall:?} candidate_wall_raw_ns={candidate_wall:?} baseline_rss_raw_bytes={baseline_rss:?} candidate_rss_raw_bytes={candidate_rss:?} baseline_commit_raw_bytes={baseline_commit:?} candidate_commit_raw_bytes={candidate_commit:?} pack_hash={} delta_hash={:?} os={} arch={} package_version={} logical_workers={} cpu_id={:?} storage_root={:?}",
        workload.label(),
        binaries.matched_source_id,
        binaries.baseline_hash,
        binaries.candidate_hash,
        baseline[0].pack_hash.as_str(),
        baseline[0].delta_hash.as_deref(),
        std::env::consts::OS,
        std::env::consts::ARCH,
        env!("CARGO_PKG_VERSION"),
        std::thread::available_parallelism().map_or(0, |threads| threads.get()),
        std::env::var("PROCESSOR_IDENTIFIER").ok(),
        &corpus.root,
    );
    assert!(
        candidate_p95.saturating_mul(100) <= baseline_p95.saturating_mul(105),
        "CLI wall P95 exceeded 105% of baseline"
    );
    assert!(
        candidate_p99.saturating_mul(100) <= baseline_p99.saturating_mul(110),
        "CLI wall P99 exceeded 110% of baseline"
    );
    assert!(
        complete_memory_peak,
        "final process memory counters unavailable; RSS gate remains open"
    );
    assert!(
        candidate_rss_p95.saturating_mul(100) <= baseline_rss_p95.saturating_mul(110),
        "CLI peak working-set P95 exceeded 110% of baseline"
    );
}

#[repr(C)]
#[derive(Default)]
struct ProcessMemoryCounters {
    cb: u32,
    page_fault_count: u32,
    peak_working_set_size: usize,
    working_set_size: usize,
    quota_peak_paged_pool_usage: usize,
    quota_paged_pool_usage: usize,
    quota_peak_non_paged_pool_usage: usize,
    quota_non_paged_pool_usage: usize,
    pagefile_usage: usize,
    peak_pagefile_usage: usize,
}

#[link(name = "kernel32")]
unsafe extern "system" {
    #[link_name = "K32GetProcessMemoryInfo"]
    fn get_process_memory_info(
        process: *mut c_void,
        counters: *mut ProcessMemoryCounters,
        size: u32,
    ) -> i32;
}

fn process_memory(child: &Child) -> Option<ProcessMemoryCounters> {
    let mut counters = ProcessMemoryCounters {
        cb: std::mem::size_of::<ProcessMemoryCounters>() as u32,
        ..ProcessMemoryCounters::default()
    };
    let size = counters.cb;
    // SAFETY: Child owns this live process handle and counters describes writable storage.
    let succeeded = unsafe { get_process_memory_info(child.as_raw_handle(), &mut counters, size) };
    (succeeded != 0).then_some(counters)
}
