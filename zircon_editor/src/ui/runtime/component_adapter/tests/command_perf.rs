use std::hint::black_box;
use std::time::Instant;

use super::*;

const SAMPLE_PAIRS: usize = 17;
const RESULTS_PER_SAMPLE: usize = 262_144;

#[test]
fn optimization_batch_fk_editor397_command_result_strings_preserve_bytes() {
    for command_id in ["save", "editor.scene.frame_selection", "plugin.command/42"] {
        assert_eq!(
            command_result_strings(command_id),
            legacy_command_result_strings(command_id)
        );
    }
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_fk_editor397_direct_command_result_strings_benchmark() {
    const COMMAND_ID: &str = "editor.scene.frame_selected_entities";
    for _ in 0..4 {
        black_box(measure(legacy_command_result_strings, COMMAND_ID));
        black_box(measure(command_result_strings, COMMAND_ID));
    }
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        if pair_index % 2 == 0 {
            legacy_samples.push(measure(legacy_command_result_strings, COMMAND_ID));
            optimized_samples.push(measure(command_result_strings, COMMAND_ID));
        } else {
            optimized_samples.push(measure(command_result_strings, COMMAND_ID));
            legacy_samples.push(measure(legacy_command_result_strings, COMMAND_ID));
        }
    }

    report_performance(&legacy_samples, &optimized_samples);
}

fn command_result_strings(command_id: &str) -> [String; 2] {
    [command_transaction(command_id), command_status(command_id)]
}

fn legacy_command_result_strings(command_id: &str) -> [String; 2] {
    [
        format!("command:{command_id}"),
        format!("Executed editor command `{command_id}`"),
    ]
}

fn measure(mut build: impl FnMut(&str) -> [String; 2], command_id: &str) -> u128 {
    let started = Instant::now();
    let mut checksum = 0_usize;
    for _ in 0..RESULTS_PER_SAMPLE {
        let strings = black_box(build(black_box(command_id)));
        checksum = checksum.wrapping_add(strings[0].len() + strings[1].len());
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn report_performance(legacy_samples: &[u128], optimized_samples: &[u128]) {
    let legacy_p95 = nearest_rank_p95(legacy_samples);
    let optimized_p95 = nearest_rank_p95(optimized_samples);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "EDITOR397_DIRECT_COMMAND_RESULT_STRINGS_BENCH_V1 sample_pairs={SAMPLE_PAIRS} results_per_sample={RESULTS_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=25",
        csv(legacy_samples),
        csv(optimized_samples),
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(75),
        "optimized p95 {optimized_p95}ns must be at most 75% of legacy p95 {legacy_p95}ns"
    );
}

fn nearest_rank_p95(samples: &[u128]) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * 95).div_ceil(100).saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
