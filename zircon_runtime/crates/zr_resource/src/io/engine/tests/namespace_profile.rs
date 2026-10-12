use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use super::*;

const SAMPLE_COUNT: usize = 31;
const PATH_COUNTS: [usize; 4] = [1, 100, 1_000, 10_000];
const SHALLOW_DEPTH: usize = 2;
const DEEP_DEPTH: usize = 16;

/// R7-C admission-only measurements. The test is explicit because it is a profiling input, not a
/// correctness gate; WPR/ETW must wrap the managed Cargo process for CPU, I/O, RSS, and power.
#[ignore = "R7-C profiling harness; run explicitly with a managed Windows Cargo lane"]
#[test]
fn namespace_admission_profile_emits_scale_matrix() {
    let sample_count = std::env::var("ZR_NAMESPACE_PROFILE_SAMPLES")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(SAMPLE_COUNT)
        .max(SAMPLE_COUNT);
    let output_root = std::env::var_os("ZIRCON_TEST_OUTPUT_ROOT")
        .or_else(|| std::env::var_os("CARGO_TARGET_DIR"))
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap().join("target"));
    let root = output_root.join("zircon-test-output").join(format!(
        "zircon-namespace-profile-{}-{}",
        std::process::id(),
        crate::io::next_test_output_id()
    ));
    let report_path = output_root.join(format!(
        "zircon-namespace-profile-report-{}.txt",
        std::process::id()
    ));
    let mut report_lines = Vec::with_capacity(16);
    fs::create_dir_all(&root).unwrap();

    for depth in [SHALLOW_DEPTH, DEEP_DEPTH] {
        for collision in [false, true] {
            for path_count in PATH_COUNTS {
                // A single live path cannot contain an ancestor/descendant pair. Keep the
                // one-path sample in the no-conflict bucket so the matrix remains truthful.
                let effective_collision = collision && path_count > 1;
                let scenario = if effective_collision {
                    "last_ancestor_collision"
                } else {
                    "no_conflict"
                };
                let scenario_root = root.join(format!("{scenario}-{depth}-{path_count}"));
                let payload_root = create_payload_root(&scenario_root, depth);
                let journal = scenario_root.join("journal");
                fs::create_dir_all(&journal).unwrap();

                let mut samples = Vec::with_capacity(sample_count);
                for _ in 0..sample_count {
                    let writes = profile_writes(&payload_root, path_count, effective_collision);
                    let started = Instant::now();
                    let result = super::validate_inputs(&journal, "profile", writes);
                    let elapsed = started.elapsed().as_nanos();
                    if effective_collision {
                        assert!(
                            result.is_err(),
                            "ancestor collision must be rejected during admission"
                        );
                    } else {
                        assert!(
                            result.is_ok(),
                            "non-conflicting profile input must validate"
                        );
                    }
                    samples.push(elapsed);
                }

                samples.sort_unstable();
                let p50 = percentile(&samples, 50);
                let p95 = percentile(&samples, 95);
                let mad = median_absolute_deviation(&samples, p50);
                let line = format!(
                    "ZR_NAMESPACE_PROFILE_V1 scenario={scenario} path_count={path_count} depth={depth} samples={sample_count} p50_ns={p50} p95_ns={p95} mad_ns={mad} metadata_queries=unavailable allocations=unavailable rss=unavailable power=unavailable"
                );
                println!("{line}");
                report_lines.push(line);
                fs::remove_dir_all(scenario_root).unwrap();
            }
        }
    }
    fs::write(&report_path, report_lines.join("\n") + "\n").unwrap();
    println!("ZR_NAMESPACE_PROFILE_REPORT path={}", report_path.display());
    fs::remove_dir_all(root).unwrap();
}

fn create_payload_root(scenario_root: &Path, depth: usize) -> PathBuf {
    let mut payload_root = scenario_root.join("payload");
    for level in 0..depth {
        payload_root.push(format!("level-{level:02}"));
    }
    fs::create_dir_all(&payload_root).unwrap();
    payload_root
}

fn profile_writes(
    payload_root: &Path,
    path_count: usize,
    collision: bool,
) -> Vec<PreparedFileWrite> {
    let mut writes = Vec::with_capacity(path_count);
    for index in 0..path_count {
        let path = if collision && index + 1 == path_count {
            payload_root.to_path_buf()
        } else {
            payload_root.join(format!("asset-{index:05}.zmeta"))
        };
        writes.push(PreparedFileWrite::new(path, Vec::new()));
    }
    writes
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let index = samples
        .len()
        .saturating_mul(percentile)
        .saturating_div(100)
        .min(samples.len().saturating_sub(1));
    samples[index]
}

fn median_absolute_deviation(samples: &[u128], median: u128) -> u128 {
    let mut deviations = samples
        .iter()
        .map(|sample| sample.abs_diff(median))
        .collect::<Vec<_>>();
    deviations.sort_unstable();
    deviations[deviations.len() / 2]
}
