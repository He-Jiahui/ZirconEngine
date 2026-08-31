use std::hint::black_box;
use std::path::PathBuf;
use std::time::Instant;

use super::{export_job_snapshot_capacity, DesktopExportJobPhase, DesktopExportJobQueue};
use crate::core::jobs::test_job_system;
use zircon_runtime::asset::project::ProjectManifest;
use zircon_runtime::asset::AssetUri;

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 2_048;
const PENDING_JOBS_PER_BUILD: usize = 255;
const SNAPSHOTS_PER_BUILD: usize = PENDING_JOBS_PER_BUILD + 1;

#[test]
fn optimization_batch_20260826fe_editor146_capacity_preserves_export_job_snapshots() {
    let mut queue = DesktopExportJobQueue::new(test_job_system());
    for index in 0..SNAPSHOTS_PER_BUILD {
        queue.enqueue(
            format!("profile-{index:03}"),
            PathBuf::from("Project"),
            ProjectManifest::new(
                format!("Project {index}"),
                AssetUri::parse("res://main.scene.toml").expect("valid test scene URI"),
                1,
            ),
            PathBuf::from(format!("Builds/{index:03}")),
        );
    }

    let snapshots = queue.snapshots();

    assert_eq!(snapshots.len(), SNAPSHOTS_PER_BUILD);
    assert!(snapshots.capacity() >= SNAPSHOTS_PER_BUILD);
    assert_eq!(snapshots[0].id, 1);
    assert_eq!(
        snapshots[SNAPSHOTS_PER_BUILD - 1].id,
        SNAPSHOTS_PER_BUILD as u64
    );
    assert_eq!(snapshots[0].phase, DesktopExportJobPhase::Queued);
    assert_eq!(export_job_snapshot_capacity(0, false), 0);
    assert_eq!(export_job_snapshot_capacity(5, true), 6);
    assert_eq!(export_job_snapshot_capacity(usize::MAX, true), usize::MAX);
}

#[test]
fn optimization_batch_20260826fe_editor146_export_job_snapshots_reserve_pending_and_active() {
    let source = include_str!("../queries.rs");
    assert!(source.contains("fn export_job_snapshot_capacity("));
    assert!(source.contains("pending_count.saturating_add(usize::from(has_active))"));
    assert!(source.contains("Vec::with_capacity(export_job_snapshot_capacity("));
    assert!(source.contains("self.pending.len()"));
    assert!(source.contains("self.active.is_some()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826fe_editor146_export_job_snapshot_capacity_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false));
            optimized_samples.push(measure(true));
        } else {
            optimized_samples.push(measure(true));
            legacy_samples.push(measure(false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR146_EXPORT_JOB_SNAPSHOT_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} pending_jobs={PENDING_JOBS_PER_BUILD} \
active_jobs=1 snapshots_per_build={SNAPSHOTS_PER_BUILD} legacy_reservations_per_build=0 \
optimized_reservations_per_build=1 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let mut snapshots = if reserve {
            Vec::with_capacity(SNAPSHOTS_PER_BUILD)
        } else {
            Vec::new()
        };
        snapshots.push(black_box(usize::MAX));
        for snapshot in 0..PENDING_JOBS_PER_BUILD {
            snapshots.push(black_box(snapshot));
        }
        checksum ^= black_box(snapshots.len() ^ snapshots.capacity());
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
