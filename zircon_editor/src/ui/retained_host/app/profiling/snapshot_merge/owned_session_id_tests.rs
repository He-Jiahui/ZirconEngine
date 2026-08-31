use std::hint::black_box;
use std::time::Instant;

use super::*;

const TARGET_P95_PERCENT: u128 = 70;

#[test]
fn optimization_batch_20260828ib_editor_profile_session_merge_reuses_buffer() {
    let mut equal = String::with_capacity(8 * 1024);
    equal.push_str(&"session".repeat(512));
    let equal_runtime = equal.clone();
    let equal_allocation = equal.as_ptr();

    merge_session_id_in_place(&mut equal, &equal_runtime);

    assert_eq!(equal, equal_runtime);
    assert_eq!(equal.as_ptr(), equal_allocation);

    let mut different = String::with_capacity(16 * 1024);
    different.push_str("editor");
    let different_allocation = different.as_ptr();
    merge_session_id_in_place(&mut different, "runtime");
    assert_eq!(different, "editor+runtime");
    assert_eq!(different.as_ptr(), different_allocation);
}

#[test]
fn optimization_batch_20260828ib_editor_profile_merge_updates_session_id_in_place() {
    let source = include_str!("../snapshot_merge.rs");
    let merge = source
        .split("pub(super) fn merge_profile_snapshot")
        .nth(1)
        .and_then(|body| body.split("fn has_profile_samples").next())
        .expect("profile snapshot merge implementation");
    let session_merge = source
        .split("fn merge_session_id_in_place")
        .nth(1)
        .and_then(|body| body.split("#[cfg(test)]").next())
        .expect("in-place session ID merge implementation");

    assert!(merge.contains("merge_session_id_in_place("));
    assert!(!merge.contains("editor_profile.session_id ="));
    assert!(session_merge.contains("editor_session_id != runtime_session_id"));
    assert!(!session_merge.contains("editor_session_id.to_string()"));
}

#[test]
#[ignore = "release performance contract; run through the managed validation coordinator"]
fn optimization_batch_20260828ib_editor_in_place_profile_session_id_benchmark() {
    const SAMPLES: usize = 11;
    const ITERATIONS: usize = 512;
    let session_id = "profile-session/".repeat(4 * 1024);

    black_box(legacy_merged_session_id(&session_id, &session_id));
    let mut warm_session = session_id.clone();
    merge_session_id_in_place(&mut warm_session, &session_id);
    black_box(warm_session);

    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);
    for sample_index in 0..SAMPLES {
        let measure_legacy = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(legacy_merged_session_id(
                    black_box(&session_id),
                    black_box(&session_id),
                ));
            }
            started.elapsed().as_nanos()
        };
        let measure_optimized = || {
            let mut editor_session_id = session_id.clone();
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                merge_session_id_in_place(
                    black_box(&mut editor_session_id),
                    black_box(&session_id),
                );
            }
            black_box(editor_session_id);
            started.elapsed().as_nanos()
        };
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_legacy());
            optimized_samples.push(measure_optimized());
        } else {
            optimized_samples.push(measure_optimized());
            legacy_samples.push(measure_legacy());
        }
    }

    let legacy_p95_ns = nearest_rank_percentile(&legacy_samples, 95);
    let optimized_p95_ns = nearest_rank_percentile(&optimized_samples, 95);
    println!(
        "EDITOR220_IN_PLACE_PROFILE_SESSION_ID_BENCH_V1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100)
            <= legacy_p95_ns.saturating_mul(TARGET_P95_PERCENT),
        "optimized P95 {optimized_p95_ns}ns must be at most {TARGET_P95_PERCENT}% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn legacy_merged_session_id(editor_session_id: &str, runtime_session_id: &str) -> String {
    if editor_session_id == runtime_session_id {
        editor_session_id.to_string()
    } else {
        format!("{editor_session_id}+{runtime_session_id}")
    }
}

fn nearest_rank_percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    ordered[(ordered.len() * percentile).div_ceil(100) - 1]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
