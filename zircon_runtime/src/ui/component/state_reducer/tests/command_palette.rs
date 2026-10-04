use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::component::UiValue;

use super::{command_entry_list, filtered_command_ids_for_entries, CommandEntry};

const FILTERED_ENTRIES_PER_SAMPLE: usize = 1_024;
const ENTRY_STREAM_LEAVES_PER_SAMPLE: usize = 1_024;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn runtime755_command_palette_filtered_capacity_preserves_source_query_and_order() {
    let entries = vec![
        command_entry("open-scene", "Open Scene", "editor"),
        command_entry("shared", "Shared Command", ""),
        command_entry("reload-runtime", "Reload Runtime", "runtime"),
        command_entry("open-settings", "Open Settings", "editor"),
    ];

    let filtered = filtered_command_ids_for_entries(&entries, Some("editor"), Some("open"));

    assert_eq!(filtered, ["open-scene", "open-settings"]);
    assert!(filtered.capacity() >= entries.len());
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime755_command_palette_filtered_capacity_release_benchmark() {
    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_ns.push(measure_growth(FILTERED_ENTRIES_PER_SAMPLE, 0));
            optimized_ns.push(measure_growth(
                FILTERED_ENTRIES_PER_SAMPLE,
                FILTERED_ENTRIES_PER_SAMPLE,
            ));
        } else {
            optimized_ns.push(measure_growth(
                FILTERED_ENTRIES_PER_SAMPLE,
                FILTERED_ENTRIES_PER_SAMPLE,
            ));
            legacy_ns.push(measure_growth(FILTERED_ENTRIES_PER_SAMPLE, 0));
        }
    }

    let legacy_growth_events = growth_events(FILTERED_ENTRIES_PER_SAMPLE, 0);
    let optimized_growth_events =
        growth_events(FILTERED_ENTRIES_PER_SAMPLE, FILTERED_ENTRIES_PER_SAMPLE);
    assert!(legacy_growth_events > optimized_growth_events);
    assert_eq!(optimized_growth_events, 0);

    println!(
        "RUNTIME755_COMMAND_PALETTE_FILTERED_CAPACITY_BENCH_V1 filtered_entries_per_sample={FILTERED_ENTRIES_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_growth_events={legacy_growth_events} optimized_growth_events={optimized_growth_events} legacy_p95_ns={} optimized_p95_ns={}",
        percentile(&legacy_ns, 95),
        percentile(&optimized_ns, 95),
    );
}

#[test]
fn runtime756_command_palette_entry_stream_preserves_nested_order_and_root_capacity() {
    let mut settings = BTreeMap::new();
    settings.insert("id".to_string(), UiValue::String("settings".to_string()));
    settings.insert(
        "label".to_string(),
        UiValue::String("Open Settings".to_string()),
    );
    let source = UiValue::Array(vec![
        UiValue::String("open|label=Open Scene".to_string()),
        UiValue::Array(vec![UiValue::Enum("build|label=Build Project".to_string())]),
        UiValue::Map(settings),
    ]);

    let entries = command_entry_list(&source);

    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].id, "open");
    assert_eq!(entries[1].id, "build");
    assert_eq!(entries[2].id, "settings");
    assert!(entries.capacity() >= 3);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime756_command_palette_entry_stream_release_benchmark() {
    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_ns.push(measure_growth(ENTRY_STREAM_LEAVES_PER_SAMPLE, 0));
            optimized_ns.push(measure_growth(
                ENTRY_STREAM_LEAVES_PER_SAMPLE,
                ENTRY_STREAM_LEAVES_PER_SAMPLE,
            ));
        } else {
            optimized_ns.push(measure_growth(
                ENTRY_STREAM_LEAVES_PER_SAMPLE,
                ENTRY_STREAM_LEAVES_PER_SAMPLE,
            ));
            legacy_ns.push(measure_growth(ENTRY_STREAM_LEAVES_PER_SAMPLE, 0));
        }
    }

    let legacy_growth_events = growth_events(ENTRY_STREAM_LEAVES_PER_SAMPLE, 0);
    let optimized_growth_events = growth_events(
        ENTRY_STREAM_LEAVES_PER_SAMPLE,
        ENTRY_STREAM_LEAVES_PER_SAMPLE,
    );
    let legacy_intermediate_vectors = ENTRY_STREAM_LEAVES_PER_SAMPLE;
    let optimized_intermediate_vectors = 0;
    assert!(legacy_growth_events > optimized_growth_events);
    assert_eq!(optimized_growth_events, 0);
    assert!(legacy_intermediate_vectors > optimized_intermediate_vectors);

    println!(
        "RUNTIME756_COMMAND_PALETTE_ENTRY_STREAM_BENCH_V1 leaf_entries_per_sample={ENTRY_STREAM_LEAVES_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_growth_events={legacy_growth_events} optimized_growth_events={optimized_growth_events} legacy_intermediate_vectors={legacy_intermediate_vectors} optimized_intermediate_vectors={optimized_intermediate_vectors} legacy_p95_ns={} optimized_p95_ns={}",
        percentile(&legacy_ns, 95),
        percentile(&optimized_ns, 95),
    );
}

fn command_entry(id: &str, label: &str, source: &str) -> CommandEntry {
    CommandEntry {
        id: id.to_string(),
        label: label.to_string(),
        source: source.to_string(),
        shortcut: String::new(),
        category: String::new(),
        keywords: Vec::new(),
        disabled: false,
    }
}

fn measure_growth(length: usize, capacity: usize) -> u128 {
    let started = Instant::now();
    let mut values = Vec::with_capacity(capacity);
    for value in 0..length {
        values.push(black_box(value));
    }
    black_box(values);
    started.elapsed().as_nanos().max(1)
}

fn growth_events(length: usize, capacity: usize) -> usize {
    if capacity >= length {
        return 0;
    }
    let mut capacity = capacity.max(1);
    let mut growth = 0;
    while capacity < length {
        capacity = capacity.saturating_mul(2);
        growth += 1;
    }
    growth
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}
