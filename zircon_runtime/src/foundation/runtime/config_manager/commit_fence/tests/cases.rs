use std::collections::HashMap;
use std::hint::black_box;
use std::io;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, Weak};
use std::time::Instant;

use crate::core::{CoreError, CoreRuntime};

use super::fence::ConfigCommitFence;
use super::path_key::physical_path;
use super::registry::{
    contains_path, lock, register_path_gate, reserve_path_owner, PathCommitEpoch,
};

const BENCH_PATH_COUNT: usize = 65_536;
const BENCH_SAMPLE_PAIRS: usize = 21;
static NEXT_TEST_PATH: AtomicU64 = AtomicU64::new(1);

#[test]
fn path_commit_gate_registry_reclaims_only_after_the_last_fence_drops() {
    let path = unique_path("last-owner-reclaim");
    let identity = physical_path(&path).unwrap();
    let first = ConfigCommitFence::register(&path).unwrap();
    let second = ConfigCommitFence::register(&path).unwrap();
    assert!(contains_path(&identity));

    drop(first);
    assert!(
        contains_path(&identity),
        "a live fence must keep the shared path gate registered"
    );

    drop(second);
    assert!(
        !contains_path(&identity),
        "the last fence must reclaim its dead path key"
    );
}

#[test]
fn path_commit_epoch_exhaustion_never_reauthorizes_a_stale_fence() {
    let path = unique_path("epoch-exhaustion");
    let identity = physical_path(&path).unwrap();
    let stale = ConfigCommitFence::register(&path).unwrap();
    let (_, _, gate) = register_path_gate(&path).unwrap();
    lock(&gate).current = u64::MAX - 1;

    let latest = ConfigCommitFence::register(&path).unwrap();
    for _ in 0..2 {
        let error = ConfigCommitFence::register(&path).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::Other);
        assert!(error
            .to_string()
            .contains(identity.operation_path().to_string_lossy().as_ref()));
        assert_eq!(lock(&gate).current, u64::MAX);
    }
    let mut initialization_ran = false;
    let error = ConfigCommitFence::initialize::<(), io::Error>(&path, |_| {
        initialization_ran = true;
        Ok(())
    })
    .unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::Other);
    assert!(!initialization_ran);
    assert_eq!(lock(&gate).current, u64::MAX);

    let mut stale_commit_ran = false;
    let stale_error = stale
        .commit(|| {
            stale_commit_ran = true;
            Ok(())
        })
        .unwrap_err();
    assert_eq!(stale_error.kind(), io::ErrorKind::Interrupted);
    assert!(!stale_commit_ran);
    assert_eq!(latest.commit(|| Ok(42_u8)).unwrap(), 42);

    drop(gate);
    drop(stale);
    drop(latest);
    assert!(!contains_path(&identity));
}

#[test]
fn path_commit_failed_initialization_preserves_the_current_owner() {
    let path = unique_path("failed-initialization");
    let identity = physical_path(&path).unwrap();
    let (_, _, gate) = register_path_gate(&path).unwrap();
    let current = ConfigCommitFence::register(&path).unwrap();
    let current_epoch = lock(&gate).current;
    let mut failed_candidate = None;
    let error = ConfigCommitFence::initialize(&path, |candidate| {
        failed_candidate = Some(candidate);
        Err::<(), _>(CoreError::ThreadSpawn(
            "initialization failure fixture".to_owned(),
        ))
    })
    .unwrap()
    .unwrap_err();
    assert!(matches!(
        error,
        CoreError::ThreadSpawn(message) if message == "initialization failure fixture"
    ));
    assert_eq!(lock(&gate).current, current_epoch);
    assert_eq!(current.commit(|| Ok(1_u8)).unwrap(), 1);

    let mut panicked_candidate = None;
    let panic = catch_unwind(AssertUnwindSafe(|| {
        ConfigCommitFence::initialize::<(), io::Error>(&path, |candidate| {
            panicked_candidate = Some(candidate);
            panic!("initialization panic fixture");
        })
    }));
    assert!(panic.is_err());
    assert!(
        !gate.is_poisoned(),
        "initialization panic must unlock normally before it resumes unwinding"
    );
    assert_eq!(lock(&gate).current, current_epoch);
    assert_eq!(current.commit(|| Ok(2_u8)).unwrap(), 2);

    let replacement = ConfigCommitFence::register(&path).unwrap();
    assert_eq!(lock(&gate).current, current_epoch + 1);
    for candidate in [failed_candidate, panicked_candidate] {
        let mut commit_ran = false;
        let error = candidate
            .unwrap()
            .commit(|| {
                commit_ran = true;
                Ok(())
            })
            .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::Interrupted);
        assert!(
            !commit_ran,
            "failed candidates must stay cancelled after epoch reuse"
        );
    }
    assert_eq!(replacement.commit(|| Ok(3_u8)).unwrap(), 3);
    drop(gate);
    drop(current);
    drop(replacement);
    assert!(!contains_path(&identity));
}

#[test]
fn path_commit_initialization_holds_the_path_gate_until_epoch_publication() {
    let path = unique_path("initialization-publication");
    let identity = physical_path(&path).unwrap();
    let other_path = unique_path("initialization-other-path");
    let other_identity = physical_path(&other_path).unwrap();
    let current = ConfigCommitFence::register(&path).unwrap();
    let (replacement, other) = ConfigCommitFence::initialize(&path, |candidate| {
        let blocked = ConfigCommitFence::register(&path).unwrap_err();
        assert_eq!(blocked.kind(), io::ErrorKind::WouldBlock);
        // Registering another path also proves initialization released the global registry lock.
        let other = ConfigCommitFence::register(&other_path)?;
        Ok::<_, io::Error>((candidate, other))
    })
    .unwrap()
    .unwrap();
    let mut old_commit_ran = false;
    let error = current
        .commit(|| {
            old_commit_ran = true;
            Ok(())
        })
        .unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::Interrupted);
    assert!(!old_commit_ran);
    replacement.commit(|| Ok(())).unwrap();
    other.commit(|| Ok(())).unwrap();
    drop(current);
    drop(replacement);
    drop(other);
    assert!(!contains_path(&identity));
    assert!(!contains_path(&other_identity));
}

#[test]
fn path_commit_failed_initialization_reclaims_the_unowned_path_gate() {
    let path = unique_path("unowned-initialization");
    let identity = physical_path(&path).unwrap();
    let error = ConfigCommitFence::initialize(&path, |_| Err::<(), _>("preparation failed"))
        .unwrap()
        .unwrap_err();
    assert_eq!(error, "preparation failed");
    assert!(!contains_path(&identity));
    let panic = catch_unwind(|| {
        ConfigCommitFence::initialize::<(), io::Error>(&path, |_| {
            panic!("unowned initialization panic fixture");
        })
    });
    assert!(panic.is_err());
    assert!(!contains_path(&identity));
    let retry = ConfigCommitFence::register(&path).unwrap();
    retry.commit(|| Ok(())).unwrap();
    drop(retry);
    assert!(!contains_path(&identity));
}

#[test]
#[ignore = "managed release performance evidence"]
fn path_commit_gate_registry_reclaim_release_benchmark() {
    let paths = (0..BENCH_PATH_COUNT)
        .map(|index| PathBuf::from(format!("runtime55/config-{index:08}.json")))
        .collect::<Vec<_>>();
    let retained_path_bytes = paths
        .iter()
        .map(|path| path.as_os_str().len())
        .sum::<usize>();
    let mut legacy_ns = Vec::with_capacity(BENCH_SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(BENCH_SAMPLE_PAIRS);
    let mut legacy_final_entries = 0;
    let mut optimized_final_entries = 0;
    let mut legacy_peak_entries = 0;
    let mut optimized_peak_entries = 0;

    for sample_index in 0..BENCH_SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            let sample = measure_legacy(&paths);
            legacy_ns.push(sample.elapsed_ns);
            legacy_final_entries = sample.final_entries;
            legacy_peak_entries = sample.peak_entries;

            let sample = measure_reclaimed(&paths);
            optimized_ns.push(sample.elapsed_ns);
            optimized_final_entries = sample.final_entries;
            optimized_peak_entries = sample.peak_entries;
        } else {
            let sample = measure_reclaimed(&paths);
            optimized_ns.push(sample.elapsed_ns);
            optimized_final_entries = sample.final_entries;
            optimized_peak_entries = sample.peak_entries;

            let sample = measure_legacy(&paths);
            legacy_ns.push(sample.elapsed_ns);
            legacy_final_entries = sample.final_entries;
            legacy_peak_entries = sample.peak_entries;
        }
    }

    assert_eq!(legacy_final_entries, BENCH_PATH_COUNT);
    assert_eq!(legacy_peak_entries, BENCH_PATH_COUNT);
    assert_eq!(optimized_final_entries, 0);
    assert_eq!(optimized_peak_entries, 1);
    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    println!(
        "FOUNDATION_PATH_GATE_RECLAIM_BENCH_V1 sample_pairs={} sample_order=alternating percentile_method=nearest_rank timing_gate=diagnostic_only path_count={} legacy_final_entries={} optimized_final_entries={} legacy_peak_entries={} optimized_peak_entries={} legacy_retained_path_bytes={} optimized_retained_path_bytes=0 legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_ns={} optimized_ns={}",
        BENCH_SAMPLE_PAIRS,
        BENCH_PATH_COUNT,
        legacy_final_entries,
        optimized_final_entries,
        legacy_peak_entries,
        optimized_peak_entries,
        retained_path_bytes,
        legacy_p50_ns,
        legacy_p95_ns,
        optimized_p50_ns,
        optimized_p95_ns,
        join_samples(&legacy_ns),
        join_samples(&optimized_ns),
    );
}

#[derive(Clone, Copy)]
struct RegistrySample {
    elapsed_ns: u128,
    final_entries: usize,
    peak_entries: usize,
}

fn measure_legacy(paths: &[PathBuf]) -> RegistrySample {
    let started = Instant::now();
    let mut gates = HashMap::<PathBuf, Weak<Mutex<PathCommitEpoch>>>::new();
    let mut peak_entries = 0;
    for path in paths {
        gates.insert(path.clone(), Weak::new());
        peak_entries = peak_entries.max(gates.len());
    }
    black_box(&gates);
    RegistrySample {
        elapsed_ns: started.elapsed().as_nanos(),
        final_entries: gates.len(),
        peak_entries,
    }
}

fn measure_reclaimed(paths: &[PathBuf]) -> RegistrySample {
    let started = Instant::now();
    let mut gates = HashMap::<PathBuf, Weak<Mutex<PathCommitEpoch>>>::new();
    let mut peak_entries = 0;
    for path in paths {
        gates.insert(path.clone(), Weak::new());
        peak_entries = peak_entries.max(gates.len());
        gates.remove(path);
    }
    black_box(&gates);
    RegistrySample {
        elapsed_ns: started.elapsed().as_nanos(),
        final_entries: gates.len(),
        peak_entries,
    }
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = ordered.len().saturating_mul(percentile).div_ceil(100);
    ordered[rank.saturating_sub(1)]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn unique_path(name: &str) -> PathBuf {
    managed_test_binary_directory().join(format!(
        "zircon-runtime55-{name}-{}-{}",
        std::process::id(),
        NEXT_TEST_PATH.fetch_add(1, Ordering::Relaxed)
    ))
}

fn managed_test_binary_directory() -> std::path::PathBuf {
    use crate::asset::ProjectPaths;
    let approved = ProjectPaths::resolve_existing("E:/cargo-targets/zircon-local")
        .expect("the managed E drive fixture root must already exist");
    let display = ProjectPaths::display_path(approved.operation_path());
    assert!(
        display
            .to_str()
            .expect("managed root must be Unicode")
            .replace('\\', "/")
            .eq_ignore_ascii_case("E:/cargo-targets/zircon-local"),
        "the approved root must resolve to the physical E drive root"
    );
    let approved = ProjectPaths::resolve_identity(approved.operation_path()).unwrap();
    let executable = std::env::current_exe().unwrap();
    let binary = ProjectPaths::resolve_identity(executable.parent().unwrap()).unwrap();
    assert!(
        binary.is_within(&approved),
        "test binary must physically live under the managed E root"
    );
    binary.operation_path().to_path_buf()
}

#[test]
fn path_owner_conflict_rejects_same_and_different_runtime_before_prepare() {
    let path = unique_path("owner-conflict");
    let runtime = CoreRuntime::new();
    let other_runtime = CoreRuntime::new();
    let (identity, _, gate) = register_path_gate(&path).unwrap();
    let owner = ConfigCommitFence::initialize_owned(&path, runtime.weak(), Ok).unwrap();
    let epoch = lock(&gate).current;
    owner
        .commit(|| {
            for (core, same_runtime) in [(runtime.weak(), true), (other_runtime.weak(), false)] {
                let mut prepared = false;
                let error = ConfigCommitFence::initialize_owned(&path, core, |_| {
                    prepared = true;
                    Ok(())
                })
                .unwrap_err();
                assert!(matches!(error, CoreError::ConfigOwnerConflict {
                same_runtime: actual, ..
            } if actual == same_runtime));
                assert!(
                    !prepared,
                    "conflict must precede a busy gate and all preparation"
                );
            }
            Ok(())
        })
        .unwrap();
    assert_eq!(lock(&gate).current, epoch);
    owner.commit(|| Ok(())).unwrap();
    drop(gate);
    drop(owner);
    assert!(!contains_path(&identity));
}

#[test]
fn path_owner_failed_reservation_releases_after_error_and_panic() {
    let path = unique_path("owner-prepare-failure");
    let runtime = CoreRuntime::new();
    let (identity, _, gate) = register_path_gate(&path).unwrap();
    let epoch = lock(&gate).current;
    let mut failed = None;
    let error = ConfigCommitFence::initialize_owned(&path, runtime.weak(), |candidate| {
        failed = Some(candidate);
        Err::<(), _>(CoreError::ThreadSpawn(
            "owner preparation fixture".to_owned(),
        ))
    })
    .unwrap_err();
    assert!(matches!(error, CoreError::ThreadSpawn(_)));
    assert_eq!(lock(&gate).current, epoch);
    let mut panicked = None;
    let panic = catch_unwind(AssertUnwindSafe(|| {
        ConfigCommitFence::initialize_owned::<()>(&path, runtime.weak(), |candidate| {
            panicked = Some(candidate);
            panic!("owner preparation panic fixture");
        })
    }));
    assert!(panic.is_err());
    assert!(!gate.is_poisoned());
    assert_eq!(lock(&gate).current, epoch);
    let retry = ConfigCommitFence::initialize_owned(&path, runtime.weak(), Ok).unwrap();
    assert_eq!(lock(&gate).current, epoch + 1);
    for candidate in [failed, panicked] {
        assert_eq!(
            candidate.unwrap().commit(|| Ok(())).unwrap_err().kind(),
            io::ErrorKind::Interrupted
        );
    }
    retry.commit(|| Ok(())).unwrap();
    drop(gate);
    drop(retry);
    assert!(!contains_path(&identity));

    // The previous failed gate can disappear after a new reservation but before gate attach.
    let path = unique_path("owner-reservation-before-gate");
    let identity = physical_path(&path).unwrap();
    let mut old = None;
    ConfigCommitFence::initialize_owned(&path, runtime.weak(), |candidate| {
        old = Some(candidate);
        Err::<(), _>(CoreError::ThreadSpawn(
            "reservation race fixture".to_owned(),
        ))
    })
    .unwrap_err();
    let pending = reserve_path_owner(&identity, runtime.weak()).unwrap();
    drop(old);
    assert!(
        contains_path(&identity),
        "old gate Drop must retain the new reservation"
    );
    assert!(matches!(
        ConfigCommitFence::initialize_owned(&path, CoreRuntime::new().weak(), Ok),
        Err(CoreError::ConfigOwnerConflict {
            same_runtime: false,
            ..
        })
    ));
    drop(pending);
    assert!(!contains_path(&identity));
    let retry = ConfigCommitFence::initialize_owned(&path, runtime.weak(), Ok).unwrap();
    retry.commit(|| Ok(())).unwrap();
    drop(retry);
    assert!(!contains_path(&identity));
}

#[test]
fn path_owner_lease_outlives_runtime_and_cancel_until_the_last_fence_drops() {
    let path = unique_path("owner-last-fence");
    let identity = physical_path(&path).unwrap();
    let runtime = CoreRuntime::new();
    let weak = runtime.weak();
    let owner = ConfigCommitFence::initialize_owned(&path, weak.clone(), Ok).unwrap();
    let retained = Arc::clone(&owner);
    let other_runtime = CoreRuntime::new();
    drop(runtime);
    assert!(weak.upgrade().is_none());
    owner.cancel();
    drop(owner);
    assert!(matches!(
        ConfigCommitFence::initialize_owned(&path, other_runtime.weak(), Ok),
        Err(CoreError::ConfigOwnerConflict {
            same_runtime: false,
            ..
        })
    ));
    assert_eq!(
        retained.commit(|| Ok(())).unwrap_err().kind(),
        io::ErrorKind::Interrupted
    );
    drop(retained);
    assert!(!contains_path(&identity));
    let replacement = ConfigCommitFence::initialize_owned(&path, other_runtime.weak(), Ok).unwrap();
    replacement.commit(|| Ok(())).unwrap();
    drop(replacement);
    assert!(!contains_path(&identity));
}
