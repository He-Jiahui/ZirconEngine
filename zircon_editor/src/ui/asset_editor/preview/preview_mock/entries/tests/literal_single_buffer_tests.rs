use std::hint::black_box;
use std::time::Instant;

use super::{parse_toml_inline_value, preview_mock_inline_literal, preview_mock_literal, Value};

const ITEM_COUNT: usize = 4_096;
const SAMPLE_PAIRS: usize = 101;

#[test]
fn editor885_preview_mock_literal_single_buffer_preserves_exact_text() {
    let raw = Value::String("line\n\"quoted\"".to_string());
    assert_eq!(preview_mock_literal(&raw), "line\n\"quoted\"");
    assert_eq!(
        preview_mock_inline_literal(&raw),
        legacy_preview_mock_inline_literal(&raw)
    );

    let nested = parse_toml_inline_value(
        r#"{ zebra = [true, "line"], alpha = { gamma = false, beta = 3 } }"#,
    )
    .expect("valid nested preview value");
    assert_eq!(
        preview_mock_literal(&nested),
        "{ alpha = { beta = 3, gamma = false }, zebra = [true, \"line\"] }"
    );
    assert_eq!(
        preview_mock_literal(&nested),
        legacy_preview_mock_literal(&nested)
    );
    assert_eq!(
        preview_mock_inline_literal(&nested),
        legacy_preview_mock_inline_literal(&nested)
    );
    assert_eq!(preview_mock_literal(&Value::Array(Vec::new())), "[]");
    assert_eq!(
        preview_mock_literal(&Value::Table(toml::Table::new())),
        "{  }"
    );
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn editor885_preview_mock_literal_single_buffer_benchmark() {
    let value = Value::Array(
        (0..ITEM_COUNT)
            .map(|index| Value::Integer(index as i64))
            .collect(),
    );
    assert_eq!(
        preview_mock_inline_literal(&value),
        legacy_preview_mock_inline_literal(&value)
    );

    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(|| legacy_preview_mock_inline_literal(&value)));
            optimized.push(measure(|| preview_mock_inline_literal(&value)));
        } else {
            optimized.push(measure(|| preview_mock_inline_literal(&value)));
            legacy.push(measure(|| legacy_preview_mock_inline_literal(&value)));
        }
    }

    let legacy_p50 = percentile(&legacy, 50);
    let legacy_p95 = percentile(&legacy, 95);
    let legacy_p99 = percentile(&legacy, 99);
    let optimized_p50 = percentile(&optimized, 50);
    let optimized_p95 = percentile(&optimized, 95);
    let optimized_p99 = percentile(&optimized, 99);
    println!(
        "EDITOR885_PREVIEW_MOCK_LITERAL_SINGLE_BUFFER_BENCH_V1 sample_pairs={SAMPLE_PAIRS} items={ITEM_COUNT} legacy_child_strings={ITEM_COUNT} legacy_vector_slots={ITEM_COUNT} optimized_child_strings=0 optimized_vector_slots=0 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} legacy_p99_ns={legacy_p99} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} optimized_p99_ns={optimized_p99}"
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(110),
        "single-buffer literal formatting P95 {optimized_p95}ns must stay within 10% of collect/join formatting P95 {legacy_p95}ns"
    );
}

fn legacy_preview_mock_literal(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Boolean(value) => value.to_string(),
        Value::Array(items) => format!(
            "[{}]",
            items
                .iter()
                .map(legacy_preview_mock_inline_literal)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Value::Table(table) => {
            let mut entries = table.iter().collect::<Vec<_>>();
            entries.sort_by(|left, right| left.0.cmp(right.0));
            format!(
                "{{ {} }}",
                entries
                    .into_iter()
                    .map(|(key, value)| {
                        format!("{key} = {}", legacy_preview_mock_inline_literal(value))
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
        _ => value.to_string(),
    }
}

fn legacy_preview_mock_inline_literal(value: &Value) -> String {
    match value {
        Value::String(text) => Value::String(text.clone()).to_string(),
        Value::Boolean(value) => value.to_string(),
        Value::Array(items) => format!(
            "[{}]",
            items
                .iter()
                .map(legacy_preview_mock_inline_literal)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Value::Table(table) => {
            let mut entries = table.iter().collect::<Vec<_>>();
            entries.sort_by(|left, right| left.0.cmp(right.0));
            format!(
                "{{ {} }}",
                entries
                    .into_iter()
                    .map(|(key, value)| {
                        format!("{key} = {}", legacy_preview_mock_inline_literal(value))
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
        _ => value.to_string(),
    }
}

fn measure<T>(work: impl FnOnce() -> T) -> u128 {
    let started = Instant::now();
    black_box(work());
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}
