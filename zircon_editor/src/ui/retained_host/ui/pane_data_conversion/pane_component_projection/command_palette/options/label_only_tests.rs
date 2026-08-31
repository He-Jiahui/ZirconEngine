use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use toml::Value;

use super::{projected_command_palette_option_rows, projected_command_palette_options};

const SAMPLE_PAIRS: usize = 31;
const PROJECTIONS_PER_SAMPLE: usize = 50;
const COMMAND_COUNT: usize = 1_024;

#[test]
fn optimization_batch_20260829aq_editor262_label_only_projection_matches_combined_rows() {
    let attributes = attributes(8);
    let optimized = projected_command_palette_options("command-palette", &attributes).unwrap();
    let legacy = projected_command_palette_option_rows("command-palette", &attributes)
        .map(|(options, _)| options)
        .unwrap();

    assert_eq!(optimized, legacy);
    assert_eq!(optimized.len(), 8);
}

#[test]
fn optimization_batch_20260829aq_editor262_label_entrypoint_skips_structured_projection() {
    let source = include_str!("../options.rs");
    let label_entrypoint = source
        .split("fn projected_command_palette_options")
        .nth(1)
        .expect("command palette label projection")
        .split("fn projected_command_palette_structured_options")
        .next()
        .expect("command palette label projection body");

    assert!(label_entrypoint.contains("projected_command_entries(attributes)"));
    assert!(label_entrypoint.contains(".map(|entry| entry.label)"));
    assert!(!label_entrypoint.contains("projected_command_palette_option_rows("));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829aq_editor262_label_only_command_palette_projection_bench() {
    let attributes = attributes(COMMAND_COUNT);
    assert_eq!(
        projected_command_palette_options("command-palette", &attributes),
        projected_command_palette_option_rows("command-palette", &attributes)
            .map(|(options, _)| options)
    );

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&attributes, false));
            optimized_samples.push(measure(&attributes, true));
        } else {
            optimized_samples.push(measure(&attributes, true));
            legacy_samples.push(measure(&attributes, false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR262_LABEL_ONLY_COMMAND_PALETTE_PROJECTION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
projections_per_sample={PROJECTIONS_PER_SAMPLE} commands_per_projection={COMMAND_COUNT} \
legacy_structured_rows_built_per_projection=1024 optimized_structured_rows_built_per_projection=0 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn attributes(command_count: usize) -> BTreeMap<String, Value> {
    let commands = (0..command_count)
        .map(|index| {
            Value::String(format!(
                "editor.command.{index:04}|label=Command {index:04}|shortcut=Ctrl+K"
            ))
        })
        .collect();
    let recent_commands = (0..command_count.min(256))
        .map(|index| Value::String(format!("editor.command.{index:04}")))
        .collect();
    BTreeMap::from([
        ("commands".to_string(), Value::Array(commands)),
        ("recent_commands".to_string(), Value::Array(recent_commands)),
        (
            "selected_command_id".to_string(),
            Value::String("editor.command.0512".to_string()),
        ),
        ("focused_index".to_string(), Value::Integer(512)),
        ("query".to_string(), Value::String("command".to_string())),
    ])
}

fn measure(attributes: &BTreeMap<String, Value>, optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..PROJECTIONS_PER_SAMPLE {
        let options = if optimized {
            projected_command_palette_options("command-palette", black_box(attributes))
        } else {
            projected_command_palette_option_rows("command-palette", black_box(attributes))
                .map(|(options, _)| options)
        }
        .expect("benchmark command palette options");
        checksum = checksum.wrapping_add(options.iter().map(String::len).sum::<usize>());
        black_box(options);
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
