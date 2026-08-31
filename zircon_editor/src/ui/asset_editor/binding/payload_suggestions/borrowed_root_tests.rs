use std::hint::black_box;
use std::time::Instant;

use super::*;

const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_20260826at_payload_suggestions_preserve_nested_last_root() {
    let root_suggestions = vec![
        ("payload".to_string(), Value::String("stale".to_string())),
        (
            "payload".to_string(),
            table([(
                "items",
                Value::Array(vec![
                    Value::String("first".to_string()),
                    Value::String("second".to_string()),
                ]),
            )]),
        ),
    ];
    let current_payload = table([(
        "payload",
        table([(
            "items",
            Value::Array(vec![
                Value::Integer(1),
                Value::Integer(2),
                Value::Integer(3),
            ]),
        )]),
    )]);

    let suggestions = contextual_binding_payload_suggestions(
        &root_suggestions,
        &current_payload,
        Some("payload.items"),
    )
    .expect("nested suggestions");

    assert_eq!(
        suggestions,
        vec![
            ("[0]".to_string(), Value::String("first".to_string())),
            ("[1]".to_string(), Value::String("second".to_string())),
            ("[3]".to_string(), Value::String("first".to_string())),
        ]
    );
    assert!(contextual_binding_payload_suggestions(
        &root_suggestions,
        &current_payload,
        Some("[0]")
    )
    .is_none());
}

#[test]
fn optimization_batch_20260826at_payload_suggestions_borrow_root_values() {
    let source = include_str!("../payload_suggestions.rs");
    let contextual = bounded_source(
        source,
        "pub(super) fn contextual_binding_payload_suggestions",
        "fn borrowed_suggestion_value",
    );
    let borrowed = bounded_source(
        source,
        "fn borrowed_suggestion_value",
        "fn immediate_nested_suggestions",
    );

    assert!(!contextual.contains("Value::Table("));
    assert!(!contextual.contains("iter().cloned().collect"));
    assert!(contextual.contains("borrowed_suggestion_value"));
    assert!(borrowed.contains("path.split_first()"));
    assert!(borrowed.contains("root_suggestions"));
    assert!(borrowed.contains(".iter()"));
    assert!(borrowed.contains(".rev()"));
    assert!(borrowed.contains("get_value_at_path(root_value, tail)"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826at_payload_suggestions_borrowed_root_p95() {
    const ROOTS: usize = 4_096;
    const BUILDS: usize = 4;
    let root_suggestions = (0..ROOTS)
        .map(|index| {
            (
                format!("root_{index:04}"),
                table([(
                    "items",
                    Value::Array(
                        (0..4)
                            .map(|item| {
                                Value::String(format!("{}-{index:04}-{item}", "payload".repeat(8)))
                            })
                            .collect(),
                    ),
                )]),
            )
        })
        .collect::<Vec<_>>();
    let current_payload = Value::Table(Default::default());
    let selected_path = "root_0000.items";

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_ns.push(measure_ns(BUILDS, || {
                legacy_suggestion_checksum(&root_suggestions, &current_payload, selected_path)
            }));
            optimized_ns.push(measure_ns(BUILDS, || {
                optimized_suggestion_checksum(&root_suggestions, &current_payload, selected_path)
            }));
        } else {
            optimized_ns.push(measure_ns(BUILDS, || {
                optimized_suggestion_checksum(&root_suggestions, &current_payload, selected_path)
            }));
            legacy_ns.push(measure_ns(BUILDS, || {
                legacy_suggestion_checksum(&root_suggestions, &current_payload, selected_path)
            }));
        }
    }

    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    assert!(
        optimized_p95_ns.saturating_mul(2) <= legacy_p95_ns,
        "borrowed payload root P95 must be at least 50% below full root projection: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );

    println!(
        "EDITOR23_PAYLOAD_SUGGESTIONS_BORROWED_ROOT_BENCH_V1 roots={ROOTS} builds_per_sample={BUILDS} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_root_table_instances_per_sample={BUILDS} optimized_root_table_instances_per_sample=0 legacy_root_entry_clones_per_sample={} optimized_root_entry_clones_per_sample=0 legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        ROOTS * BUILDS,
        join_samples(&legacy_ns),
        join_samples(&optimized_ns),
    );
}

fn optimized_suggestion_checksum(
    root_suggestions: &[(String, Value)],
    current_payload: &Value,
    selected_path: &str,
) -> usize {
    contextual_binding_payload_suggestions(
        black_box(root_suggestions),
        current_payload,
        Some(selected_path),
    )
    .expect("optimized suggestions")
    .len()
}

fn legacy_suggestion_checksum(
    root_suggestions: &[(String, Value)],
    current_payload: &Value,
    selected_payload_key: &str,
) -> usize {
    let selected_path = parse_value_path(selected_payload_key).expect("selected path");
    let suggestion_root = Value::Table(black_box(root_suggestions).iter().cloned().collect());
    let selected_value =
        get_value_at_path(&suggestion_root, &selected_path).expect("legacy selected suggestion");
    let current_selected_value = get_value_at_path(current_payload, &selected_path);
    immediate_nested_suggestions(selected_value, current_selected_value).len()
}

fn table<const N: usize>(entries: [(&str, Value); N]) -> Value {
    Value::Table(
        entries
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect(),
    )
}

fn measure_ns(iterations: usize, mut operation: impl FnMut() -> usize) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..iterations {
        checksum = checksum.wrapping_add(black_box(operation()));
    }
    black_box(checksum);
    started.elapsed().as_nanos()
}

fn bounded_source<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split(start)
        .nth(1)
        .expect("source start")
        .split(end)
        .next()
        .expect("source end")
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
