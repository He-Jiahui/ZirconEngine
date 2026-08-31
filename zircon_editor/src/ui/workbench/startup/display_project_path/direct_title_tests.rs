use std::hint::black_box;
use std::time::Instant;

use super::project_title_from_display_path;

const OPERATIONS_PER_SAMPLE: usize = 4_096;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn optimization_batch_20260826hd_editor196_preserves_project_title_contract() {
    assert_eq!(
        project_title_from_display_path("C:\\projects\\Zircon".to_string()),
        "Zircon"
    );
    assert_eq!(
        project_title_from_display_path("/projects/Zircon///".to_string()),
        "Zircon"
    );
    assert_eq!(
        project_title_from_display_path("projects\\nested/Project".to_string()),
        "Project"
    );
    assert_eq!(project_title_from_display_path("///".to_string()), "///");
}

#[test]
fn optimization_batch_20260826hd_editor196_splits_without_path_replacement() {
    let source = include_str!("../display_project_path.rs");
    let start = source
        .find("fn project_title_from_display_path(")
        .expect("project_title_from_display_path function");
    let end = source[start..]
        .find("\n#[cfg(test)]")
        .map(|offset| start + offset)
        .expect("test module boundary");
    let body = &source[start..end];

    assert!(body.contains("trim_end_matches(['/', '\\\\'])"));
    assert!(body.contains("rsplit(['/', '\\\\'])"));
    assert!(!body.contains("replace("));
}

#[test]
#[ignore = "managed release performance evidence"]
fn optimization_batch_20260826hd_editor196_project_title_direct_split_release_benchmark() {
    let display_path = format!(
        "C:\\{}\\ZirconProject\\",
        (0..96)
            .map(|index| format!("workspace-segment-{index:03}"))
            .collect::<Vec<_>>()
            .join("\\")
    );
    assert_eq!(
        project_title_from_display_path(display_path.clone()),
        legacy_project_title_from_display_path(display_path.clone())
    );

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        let legacy_inputs = (0..OPERATIONS_PER_SAMPLE)
            .map(|_| display_path.clone())
            .collect::<Vec<_>>();
        let optimized_inputs = (0..OPERATIONS_PER_SAMPLE)
            .map(|_| display_path.clone())
            .collect::<Vec<_>>();
        let measure_legacy = || {
            let started = Instant::now();
            for path in legacy_inputs {
                black_box(legacy_project_title_from_display_path(path));
            }
            started.elapsed().as_nanos().max(1)
        };
        let measure_optimized = || {
            let started = Instant::now();
            for path in optimized_inputs {
                black_box(project_title_from_display_path(path));
            }
            started.elapsed().as_nanos().max(1)
        };
        if sample_index % 2 == 0 {
            legacy_ns.push(measure_legacy());
            optimized_ns.push(measure_optimized());
        } else {
            optimized_ns.push(measure_optimized());
            legacy_ns.push(measure_legacy());
        }
    }

    let legacy_p50_ns = percentile(&legacy_ns, 50);
    let legacy_p95_ns = percentile(&legacy_ns, 95);
    let optimized_p50_ns = percentile(&optimized_ns, 50);
    let optimized_p95_ns = percentile(&optimized_ns, 95);
    println!(
        "EDITOR196_PROJECT_TITLE_DIRECT_SPLIT_BENCH_V1 input_bytes={} \
         operations_per_sample={OPERATIONS_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} \
         legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} \
         optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} \
         legacy_ns={} optimized_ns={}",
        display_path.len(),
        samples(&legacy_ns),
        samples(&optimized_ns),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "optimized P95 {optimized_p95_ns}ns must be at most 70% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn legacy_project_title_from_display_path(display_path: String) -> String {
    let normalized = display_path.replace('\\', "/");
    let trimmed = normalized.trim_end_matches('/');
    let title = trimmed
        .rsplit('/')
        .find(|segment| !segment.trim().is_empty())
        .unwrap_or(trimmed);
    if title.is_empty() {
        display_path
    } else {
        title.to_string()
    }
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = ordered.len().saturating_mul(percentile).div_ceil(100);
    ordered[rank.saturating_sub(1)]
}

fn samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
