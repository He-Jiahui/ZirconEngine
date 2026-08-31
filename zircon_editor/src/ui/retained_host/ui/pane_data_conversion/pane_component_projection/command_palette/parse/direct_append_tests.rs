use std::hint::black_box;
use std::time::Instant;

use toml::Value;

use super::{command_entry_from_string, command_entry_from_table, command_entry_list};
use crate::ui::retained_host::ui::pane_data_conversion::pane_component_projection::command_palette::entry::CommandProjectionEntry;

const SAMPLE_PAIRS: usize = 21;
const ITERATIONS: usize = 16;
const WIDTH: usize = 4;
const DEPTH: usize = 5;
const ENTRIES_PER_PARSE: usize = WIDTH.pow(DEPTH as u32);
const LEGACY_VECS_PER_PARSE: usize = (WIDTH.pow((DEPTH + 1) as u32) - 1) / (WIDTH - 1);

#[test]
fn optimization_batch_20260826db_editor91_command_entries_preserve_nested_order() {
    let value = Value::Array(vec![
        Value::String(" first |label=First".to_string()),
        Value::Array(vec![
            Value::Boolean(true),
            Value::String("second|shortcut=Ctrl+2".to_string()),
        ]),
        Value::String("   ".to_string()),
        Value::String("third|disabled=yes".to_string()),
    ]);

    let entries = command_entry_list(&value);
    let projected = entries
        .iter()
        .map(|entry| {
            (
                entry.id.as_str(),
                entry.label.as_str(),
                entry.description.as_str(),
                entry.disabled,
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(
        projected,
        [
            ("first", "First", "", false),
            ("second", "second", "Ctrl+2", false),
            ("third", "third", "", true),
        ]
    );
}

#[test]
fn optimization_batch_20260826db_editor91_command_entries_use_one_output_collector() {
    let source = include_str!("../parse.rs");

    assert!(source.contains("append_command_entries(value, &mut entries)"));
    assert!(source.contains("append_command_entries(value, entries)"));
    assert!(!source.contains("values.iter().flat_map(command_entry_list).collect()"));
    assert!(!source.contains("command_entry_from_string(value).into_iter().collect()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826db_editor91_command_entry_direct_append_bench() {
    let fixture = nested_fixture(DEPTH, 0);
    assert_eq!(command_entry_list(&fixture).len(), ENTRIES_PER_PARSE);

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        let measure_legacy = || measure(&fixture, legacy_command_entry_list);
        let measure_optimized = || measure(&fixture, command_entry_list);
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy());
            optimized_samples.push(measure_optimized());
        } else {
            optimized_samples.push(measure_optimized());
            legacy_samples.push(measure_legacy());
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR91_COMMAND_ENTRY_DIRECT_APPEND_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
iterations={ITERATIONS} entries_per_parse={ENTRIES_PER_PARSE} \
legacy_vec_instances_per_parse={LEGACY_VECS_PER_PARSE} optimized_vec_instances_per_parse=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "direct append P95 {optimized_p95_ns}ns must be at most 70% of recursive collectors P95 {legacy_p95_ns}ns"
    );
}

fn nested_fixture(depth: usize, seed: usize) -> Value {
    if depth == 0 {
        return Value::String(format!(
            "command.{seed:05}|label=Command {seed:05}|shortcut=Ctrl+K"
        ));
    }
    Value::Array(
        (0..WIDTH)
            .map(|index| nested_fixture(depth - 1, seed * WIDTH + index))
            .collect(),
    )
}

fn legacy_command_entry_list(value: &Value) -> Vec<CommandProjectionEntry> {
    match value {
        Value::Array(values) => values.iter().flat_map(legacy_command_entry_list).collect(),
        Value::String(value) => command_entry_from_string(value).into_iter().collect(),
        Value::Table(values) => command_entry_from_table(values).into_iter().collect(),
        _ => Vec::new(),
    }
}

fn measure(fixture: &Value, parse: fn(&Value) -> Vec<CommandProjectionEntry>) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..ITERATIONS {
        checksum ^= black_box(parse(black_box(fixture))).len();
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
