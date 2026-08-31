use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::core::framework::ai::{AiDecisionStatus, AiPerceptionSense};

use super::{parse_parallel_policy, parse_perception_sense, parse_task_result, ParallelPolicy};

const BENCHMARK_PARSE_COUNT: usize = 4_096;
const BENCHMARK_SAMPLE_COUNT: usize = 21;
const TASK_RESULT_INPUTS: &[&str] = &[
    " idle ",
    "RUNNING",
    "In_Progress",
    "inprogress",
    " Succeeded ",
    "SUCCESS",
    "succeed",
    "FAILED",
    " failure ",
    "Fail",
    "BLOCKED",
    "unknown",
];

#[test]
fn borrowed_parameter_parsers_preserve_alias_case_and_trim_semantics() {
    assert_eq!(parse_task_result(" IDLE "), Some(AiDecisionStatus::Idle));
    assert_eq!(
        parse_task_result(" In_Progress "),
        Some(AiDecisionStatus::Running)
    );
    assert_eq!(
        parse_task_result("SUCCESS"),
        Some(AiDecisionStatus::Succeeded)
    );
    assert_eq!(
        parse_task_result(" Failure "),
        Some(AiDecisionStatus::Failed)
    );
    assert_eq!(
        parse_task_result("BLOCKED"),
        Some(AiDecisionStatus::Blocked)
    );
    assert_eq!(parse_task_result("other"), None);

    assert_eq!(parse_parallel_policy(" ALL "), Some(ParallelPolicy::All));
    assert_eq!(parse_parallel_policy("Any"), Some(ParallelPolicy::Any));
    assert_eq!(parse_parallel_policy("majority"), None);

    assert_eq!(
        parse_perception_sense(" Vision "),
        Some(AiPerceptionSense::Sight)
    );
    assert_eq!(
        parse_perception_sense("SOUND"),
        Some(AiPerceptionSense::Hearing)
    );
    assert_eq!(
        parse_perception_sense("Damage"),
        Some(AiPerceptionSense::Damage)
    );
    assert_eq!(parse_perception_sense("unknown"), None);
}

#[test]
fn parameter_parsers_compare_borrowed_trimmed_values_without_normalization_strings() {
    let source = include_str!("../parameters.rs");

    assert!(source.contains("eq_ignore_ascii_case"));
    assert!(!source.contains("normalized_parameter_value"));
    assert!(!source.contains("to_ascii_lowercase"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn borrowed_parameter_enum_parsing_release_benchmark_evidence() {
    assert_eq!(legacy_checksum(), borrowed_checksum());
    let (legacy_samples, optimized_samples) =
        benchmark_paired_samples(legacy_checksum, borrowed_checksum);
    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let legacy_ns = benchmark_samples_csv(&legacy_samples);
    let optimized_ns = benchmark_samples_csv(&optimized_samples);

    println!(
        "PERF_RESULT plugins15_borrowed_parameter_enum_parsing parses={BENCHMARK_PARSE_COUNT} samples={BENCHMARK_SAMPLE_COUNT} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_string_allocations_per_sample={BENCHMARK_PARSE_COUNT} optimized_string_allocations_per_sample=0 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_ns={legacy_ns} optimized_ns={optimized_ns}"
    );
    assert!(
        optimized_p95 * 2 <= legacy_p95,
        "optimized P95 {optimized_p95}ns must be no more than 50% of legacy P95 {legacy_p95}ns"
    );
}

fn legacy_checksum() -> u64 {
    let mut checksum = 0_u64;
    for index in 0..BENCHMARK_PARSE_COUNT {
        let value = black_box(TASK_RESULT_INPUTS[index % TASK_RESULT_INPUTS.len()]);
        checksum += legacy_parse_task_result(value).map_or(0, status_code);
    }
    checksum
}

fn borrowed_checksum() -> u64 {
    let mut checksum = 0_u64;
    for index in 0..BENCHMARK_PARSE_COUNT {
        let value = black_box(TASK_RESULT_INPUTS[index % TASK_RESULT_INPUTS.len()]);
        checksum += parse_task_result(value).map_or(0, status_code);
    }
    checksum
}

fn legacy_parse_task_result(value: &str) -> Option<AiDecisionStatus> {
    match value.trim().to_ascii_lowercase().as_str() {
        "idle" => Some(AiDecisionStatus::Idle),
        "running" | "in_progress" | "inprogress" => Some(AiDecisionStatus::Running),
        "succeeded" | "success" | "succeed" => Some(AiDecisionStatus::Succeeded),
        "failed" | "failure" | "fail" => Some(AiDecisionStatus::Failed),
        "blocked" => Some(AiDecisionStatus::Blocked),
        _ => None,
    }
}

fn status_code(status: AiDecisionStatus) -> u64 {
    match status {
        AiDecisionStatus::Idle => 1,
        AiDecisionStatus::Running => 2,
        AiDecisionStatus::Succeeded => 3,
        AiDecisionStatus::Failed => 5,
        AiDecisionStatus::Blocked => 7,
    }
}

fn benchmark_paired_samples(
    mut legacy: impl FnMut() -> u64,
    mut optimized: impl FnMut() -> u64,
) -> (Vec<u128>, Vec<u128>) {
    black_box(legacy());
    black_box(optimized());
    let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
    for sample_index in 0..BENCHMARK_SAMPLE_COUNT {
        if sample_index % 2 == 0 {
            legacy_samples.push(benchmark_sample(&mut legacy));
            optimized_samples.push(benchmark_sample(&mut optimized));
        } else {
            optimized_samples.push(benchmark_sample(&mut optimized));
            legacy_samples.push(benchmark_sample(&mut legacy));
        }
    }
    (legacy_samples, optimized_samples)
}

fn benchmark_sample(operation: &mut impl FnMut() -> u64) -> u128 {
    let started = Instant::now();
    black_box(operation());
    started.elapsed().as_nanos()
}

fn benchmark_samples_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    assert!(!sorted.is_empty());
    assert!((1..=100).contains(&percentile));
    let index = (sorted.len() * percentile).div_ceil(100) - 1;
    sorted[index]
}
