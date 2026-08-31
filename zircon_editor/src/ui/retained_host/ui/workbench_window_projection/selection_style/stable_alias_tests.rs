use std::hint::black_box;
use std::time::Instant;

use super::*;

const OPERATIONS_PER_SAMPLE: usize = 8 * 1024;
const SAMPLE_PAIRS: usize = 21;
const ALIAS_KEYS: &[&str] = &[
    "background",
    "background_color",
    "border",
    "border_color",
    "foreground",
    "foreground_color",
];
const ALIAS_VALUE: &str = "#a4aeb4";

#[test]
fn optimization_batch_20260826hg_editor199_preserves_selection_alias_updates() {
    let mut values = BTreeMap::new();
    set_toml_string_aliases(&mut values, &["background", "border"], "#101820");
    assert_eq!(
        values.get("background").and_then(toml::Value::as_str),
        Some("#101820")
    );
    assert_eq!(
        values.get("border").and_then(toml::Value::as_str),
        Some("#101820")
    );

    set_toml_string_aliases(&mut values, &["background", "border"], "#2aa6b8");
    assert_eq!(
        values.get("background").and_then(toml::Value::as_str),
        Some("#2aa6b8")
    );
    assert_eq!(
        values.get("border").and_then(toml::Value::as_str),
        Some("#2aa6b8")
    );

    set_toml_string_aliases(&mut values, &["background", "border"], "#2aa6b8");
    assert_eq!(values.len(), 2);
}

#[test]
fn optimization_batch_20260826hg_editor199_skips_stable_selection_alias_values() {
    let source = include_str!("../selection_style.rs");
    let start = source
        .find("fn set_toml_string_aliases(")
        .expect("set_toml_string_aliases function");
    let end = source[start..]
        .find("\n#[cfg(test)]")
        .map(|offset| start + offset)
        .expect("test module boundary");
    let body = &source[start..end];

    assert!(body.contains("values.get_mut(*key)"));
    assert!(body.contains("current.as_str() == Some(value)"));
    assert!(body.contains("*current = toml::Value::String(value.to_string())"));
}

#[test]
#[ignore = "managed release performance evidence"]
fn optimization_batch_20260826hg_editor199_stable_selection_alias_release_benchmark() {
    let baseline = ALIAS_KEYS
        .iter()
        .map(|key| {
            (
                (*key).to_string(),
                toml::Value::String(ALIAS_VALUE.to_string()),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut legacy = baseline.clone();
    let mut optimized = baseline;

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        let mut measure_legacy = || {
            let started = Instant::now();
            for _ in 0..OPERATIONS_PER_SAMPLE {
                legacy_set_toml_string_aliases(
                    black_box(&mut legacy),
                    black_box(ALIAS_KEYS),
                    black_box(ALIAS_VALUE),
                );
            }
            legacy_ns.push(started.elapsed().as_nanos().max(1));
        };
        let mut measure_optimized = || {
            let started = Instant::now();
            for _ in 0..OPERATIONS_PER_SAMPLE {
                set_toml_string_aliases(
                    black_box(&mut optimized),
                    black_box(ALIAS_KEYS),
                    black_box(ALIAS_VALUE),
                );
            }
            optimized_ns.push(started.elapsed().as_nanos().max(1));
        };
        if sample_index % 2 == 0 {
            measure_legacy();
            measure_optimized();
        } else {
            measure_optimized();
            measure_legacy();
        }
    }
    assert_eq!(legacy, optimized);

    let legacy_p50_ns = percentile(&legacy_ns, 50);
    let legacy_p95_ns = percentile(&legacy_ns, 95);
    let optimized_p50_ns = percentile(&optimized_ns, 50);
    let optimized_p95_ns = percentile(&optimized_ns, 95);
    println!(
        "EDITOR199_STABLE_SELECTION_ALIAS_BENCH_V1 \
         alias_keys={} operations_per_sample={OPERATIONS_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} \
         legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} \
         optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} \
         legacy_ns={} optimized_ns={}",
        ALIAS_KEYS.len(),
        samples(&legacy_ns),
        samples(&optimized_ns),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "optimized P95 {optimized_p95_ns}ns must be at most 70% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn legacy_set_toml_string_aliases(
    values: &mut BTreeMap<String, toml::Value>,
    keys: &[&str],
    value: &str,
) {
    for key in keys {
        values.insert((*key).to_string(), toml::Value::String(value.to_string()));
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
