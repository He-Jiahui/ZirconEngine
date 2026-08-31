use std::hint::black_box;
use std::path::Path;
use std::time::Instant;

use zircon_runtime::asset::project::ProjectPaths;

use super::output_paths_alias;

const CHECKS_PER_SAMPLE: usize = 2048;
const SAMPLE_PAIRS: usize = 31;

fn legacy_output_paths_alias(report: &Path, artifact: &Path) -> bool {
    same_file::is_same_file(report, artifact).unwrap_or(false)
        || ProjectPaths::same_lexical_path(report, artifact).unwrap_or(false)
}

fn measure(report: &Path, artifact: &Path, optimized: bool) -> u128 {
    let started = Instant::now();
    let mut aliases = 0;
    for _ in 0..CHECKS_PER_SAMPLE {
        aliases += if optimized {
            output_paths_alias(black_box(report), black_box(artifact)) as usize
        } else {
            legacy_output_paths_alias(black_box(report), black_box(artifact)) as usize
        };
    }
    black_box(aliases);
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

#[test]
fn optimization_batch_20260829bq_runtime344_output_aliases_preserve_results() {
    for (report, artifact) in [
        ("out/report.json", "out/report.json"),
        ("out/./report.json", "out/report.json"),
        ("out/report.json", "out/contents.json"),
    ] {
        assert_eq!(
            output_paths_alias(Path::new(report), Path::new(artifact)),
            legacy_output_paths_alias(Path::new(report), Path::new(artifact)),
            "{report:?} vs {artifact:?}"
        );
    }
}

#[test]
fn optimization_batch_20260829bq_runtime344_lexical_alias_check_runs_first() {
    let source = include_str!("../args.rs");
    let production = source.split_once("#[cfg(test)]").expect("test boundary").0;
    let function = production
        .split_once("fn output_paths_alias")
        .expect("alias function")
        .1;
    let lexical = function
        .find("ProjectPaths::same_lexical_path")
        .expect("lexical check");
    let identity = function
        .find("same_file::is_same_file")
        .expect("identity check");
    assert!(lexical < identity);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829bq_runtime344_lexical_output_alias_first_bench() {
    let report = Path::new("target/optimization-bq/out/REPORT.json");
    let artifact = Path::new("target/optimization-bq/out/report.json");
    let mut baseline = Vec::with_capacity(SAMPLE_PAIRS);
    let mut candidate = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            baseline.push(measure(report, artifact, false));
            candidate.push(measure(report, artifact, true));
        } else {
            candidate.push(measure(report, artifact, true));
            baseline.push(measure(report, artifact, false));
        }
    }
    let baseline_p50_ns = percentile(&baseline, 50);
    let candidate_p50_ns = percentile(&candidate, 50);
    let baseline_p95_ns = percentile(&baseline, 95);
    let candidate_p95_ns = percentile(&candidate, 95);
    println!(
        "RUNTIME344_LEXICAL_OUTPUT_ALIAS_FIRST_BENCH_V1 sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} baseline_identity_probes={CHECKS_PER_SAMPLE} candidate_identity_probes=0 baseline_p50_ns={baseline_p50_ns} candidate_p50_ns={candidate_p50_ns} baseline_p95_ns={baseline_p95_ns} candidate_p95_ns={candidate_p95_ns} baseline_raw_ns={} candidate_raw_ns={}",
        sample_csv(&baseline),
        sample_csv(&candidate)
    );
    assert!(candidate_p95_ns.saturating_mul(100) <= baseline_p95_ns.saturating_mul(70));
}
