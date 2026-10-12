use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use super::*;

const SAMPLE_PAIRS: usize = 17;
const UPDATES_PER_SAMPLE: usize = 65_536;

#[test]
fn optimization_batch_fq_editor403_reuses_existing_showcase_attribute_keys() {
    let mut attributes = BTreeMap::new();
    set_projected_attribute(&mut attributes, "selected", TomlValue::Boolean(false));
    set_projected_attribute(&mut attributes, "selected", TomlValue::Boolean(true));

    assert_eq!(attributes.len(), 1);
    assert_eq!(attributes.get("selected"), Some(&TomlValue::Boolean(true)));
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_fq_editor403_borrowed_showcase_attribute_key_benchmark() {
    for _ in 0..4 {
        black_box(measure_updates(false));
        black_box(measure_updates(true));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_updates(false));
            optimized_samples.push(measure_updates(true));
        } else {
            optimized_samples.push(measure_updates(true));
            legacy_samples.push(measure_updates(false));
        }
    }

    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "EDITOR403_BORROWED_SHOWCASE_ATTRIBUTE_KEY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} updates_per_sample={UPDATES_PER_SAMPLE} legacy_owned_keys_per_sample={UPDATES_PER_SAMPLE} optimized_owned_keys_per_sample=0 value_strings_per_sample={UPDATES_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=25",
        csv(&legacy_samples),
        csv(&optimized_samples),
    );
    assert!(optimized_p95 <= legacy_p95 * 75 / 100);
}

fn measure_updates(optimized: bool) -> u128 {
    let mut attributes = BTreeMap::from([(
        "selection_state".to_owned(),
        TomlValue::String("normal".to_owned()),
    )]);
    let started = Instant::now();
    for update in 0..UPDATES_PER_SAMPLE {
        let value = TomlValue::String(if update & 1 == 0 {
            "selected".to_owned()
        } else {
            "normal".to_owned()
        });
        if optimized {
            set_projected_attribute(black_box(&mut attributes), "selection_state", value);
        } else {
            attributes.insert("selection_state".to_owned(), value);
        }
    }
    black_box(attributes);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
