use super::*;

#[test]
fn moved_constructed_entries_preserve_cloning_results() {
    let parser = BindingParser::new("");
    let record_cases = [
        vec![
            UiBindingValue::String("first".to_string()),
            UiBindingValue::Unsigned(1),
            UiBindingValue::String("second".to_string()),
            UiBindingValue::String("value".to_string()),
        ],
        vec![
            UiBindingValue::String("duplicate".to_string()),
            UiBindingValue::Unsigned(1),
            UiBindingValue::String("duplicate".to_string()),
            UiBindingValue::Unsigned(2),
        ],
        vec![UiBindingValue::Bool(true), UiBindingValue::Null],
        vec![UiBindingValue::String("odd".to_string())],
    ];
    for arguments in record_cases {
        assert_eq!(
            parser.construct_record("record", arguments.clone()),
            parser.construct_record_cloning("record", arguments),
        );
    }

    let map_cases = [
        vec![
            UiBindingValue::String("name".to_string()),
            UiBindingValue::String("value".to_string()),
            UiBindingValue::Unsigned(7),
            UiBindingValue::Bool(true),
            UiBindingValue::Signed(-3),
            UiBindingValue::Null,
        ],
        vec![UiBindingValue::Null, UiBindingValue::Bool(false)],
        vec![UiBindingValue::Unsigned(1)],
    ];
    for arguments in map_cases {
        assert_eq!(
            parser.construct_map("map", arguments.clone()),
            parser.construct_map_cloning("map", arguments),
        );
    }
}

fn benchmark_record_arguments(entry_count: usize) -> Vec<UiBindingValue> {
    let value = "v".repeat(128);
    let mut arguments = Vec::with_capacity(entry_count * 2);
    for index in 0..entry_count {
        arguments.push(UiBindingValue::String(format!(
            "field_{index:04}_{}",
            "k".repeat(96)
        )));
        arguments.push(UiBindingValue::String(value.clone()));
    }
    arguments
}

#[test]
#[ignore = "release-only moved constructed binding entry benchmark"]
fn runtime_interface03_batch34_moved_constructed_entry_release_benchmark() {
    use std::{hint::black_box, time::Instant};

    const ENTRY_COUNT: usize = 32;
    const LOOKUP_COUNT: usize = 512;
    const SAMPLE_COUNT: usize = 11;
    let parser = BindingParser::new("");
    let arguments = benchmark_record_arguments(ENTRY_COUNT);
    let mut cloning_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut moved_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_cloning = || {
            let started = Instant::now();
            for _ in 0..LOOKUP_COUNT {
                black_box(parser.construct_record_cloning("record", black_box(arguments.clone())));
            }
            started.elapsed().as_nanos()
        };
        let measure_moved = || {
            let started = Instant::now();
            for _ in 0..LOOKUP_COUNT {
                black_box(parser.construct_record("record", black_box(arguments.clone())));
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            cloning_samples.push(measure_cloning());
            moved_samples.push(measure_moved());
        } else {
            moved_samples.push(measure_moved());
            cloning_samples.push(measure_cloning());
        }
    }

    cloning_samples.sort_unstable();
    moved_samples.sort_unstable();
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_MOVED_CONSTRUCTED_ENTRY_BENCH_V1 entries={ENTRY_COUNT} lookups={LOOKUP_COUNT} samples={SAMPLE_COUNT} cloning_p95_ns={} moved_p95_ns={}",
        cloning_samples[p95], moved_samples[p95],
    );
    assert!(
        moved_samples[p95].saturating_mul(5) <= cloning_samples[p95].saturating_mul(4),
        "moved constructed entries must improve P95 by at least 20%: cloning={}ns moved={}ns",
        cloning_samples[p95],
        moved_samples[p95],
    );
}
