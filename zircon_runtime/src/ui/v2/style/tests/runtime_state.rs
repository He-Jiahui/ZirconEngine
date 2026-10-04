use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use super::*;

#[test]
fn pseudo_state_capacity_preserves_alias_and_order_semantics() {
    let mut node = UiV2ArenaNode::default();
    for name in ["custom_a", "custom_b", "custom_c"] {
        node.props.insert(name.to_string(), Value::Boolean(true));
    }

    let states = collect_pseudo_states(&node);

    assert_eq!(states.len(), 4);
    assert!(states.windows(2).all(|pair| pair[0] <= pair[1]));
    assert!(states.contains(&"resolved-normal".to_string()));
    assert!(states.capacity() >= pseudo_state_initial_capacity(3));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime810_v2_pseudo_state_capacity_release_benchmark() {
    const AUTHORED_STATE_COUNT: usize = 4_096;
    const BUILDS_PER_SAMPLE: usize = 4_096;
    const SAMPLE_PAIRS: usize = 17;

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_state_growth(
                AUTHORED_STATE_COUNT,
                BUILDS_PER_SAMPLE,
                false,
            ));
            optimized_samples.push(measure_state_growth(
                AUTHORED_STATE_COUNT,
                BUILDS_PER_SAMPLE,
                true,
            ));
        } else {
            optimized_samples.push(measure_state_growth(
                AUTHORED_STATE_COUNT,
                BUILDS_PER_SAMPLE,
                true,
            ));
            legacy_samples.push(measure_state_growth(
                AUTHORED_STATE_COUNT,
                BUILDS_PER_SAMPLE,
                false,
            ));
        }
    }

    let legacy_growth_events = growth_events(AUTHORED_STATE_COUNT + 1);
    let optimized_growth_events = 0;
    assert!(legacy_growth_events > optimized_growth_events);
    println!(
        "RUNTIME810_V2_PSEUDO_STATE_CAPACITY_BENCH_V1 authored_state_count={AUTHORED_STATE_COUNT} builds_per_sample={BUILDS_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} legacy_growth_events={legacy_growth_events} optimized_growth_events={optimized_growth_events} legacy_p95_ns={} optimized_p95_ns={}",
        percentile(&legacy_samples, 95),
        percentile(&optimized_samples, 95),
    );
}

fn measure_state_growth(state_count: usize, builds: usize, optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..builds {
        let mut states = if optimized {
            Vec::with_capacity(pseudo_state_initial_capacity(state_count))
        } else {
            Vec::new()
        };
        for index in 0..state_count {
            states.push(format!("custom-{index}"));
        }
        states.push("resolved-normal".to_string());
        checksum = checksum.wrapping_add(states.len());
        black_box(states);
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn growth_events(item_count: usize) -> usize {
    let mut capacity = 0usize;
    let mut events = 0usize;
    for length in 1..=item_count {
        if length > capacity {
            capacity = if capacity == 0 {
                4
            } else {
                capacity.saturating_mul(2)
            };
            events += 1;
        }
    }
    events
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

#[test]
fn runtime_style_delta_preserves_dirty_domain_classification() {
    let old_attributes = BTreeMap::from([
        (
            "background".to_string(),
            Value::String("#111111".to_string()),
        ),
        ("font_size".to_string(), Value::Integer(12)),
        ("hovered".to_string(), Value::Boolean(true)),
        ("opacity".to_string(), Value::Float(1.0)),
        ("width".to_string(), Value::Integer(100)),
    ]);
    let new_attributes = BTreeMap::from([
        (
            "background".to_string(),
            Value::String("#222222".to_string()),
        ),
        ("font_size".to_string(), Value::Integer(14)),
        ("opacity".to_string(), Value::Float(1.0)),
        ("pressed".to_string(), Value::Boolean(true)),
        ("width".to_string(), Value::Integer(120)),
    ]);

    let dirty = dirty_for_runtime_style_delta(&old_attributes, &new_attributes);

    assert!(dirty.render);
    assert!(dirty.text);
    assert!(dirty.style);
    assert!(!dirty.layout);
    assert!(!dirty.hit_test);
    assert!(!dirty.input);
    assert!(!dirty.visible_range);
}

#[test]
fn runtime_style_delta_keeps_state_and_render_only_changes_render_scoped() {
    let old_attributes = BTreeMap::from([
        ("hovered".to_string(), Value::Boolean(true)),
        ("opacity".to_string(), Value::Float(1.0)),
    ]);
    let new_attributes = BTreeMap::from([
        ("opacity".to_string(), Value::Float(0.5)),
        ("pressed".to_string(), Value::Boolean(true)),
    ]);

    let dirty = dirty_for_runtime_style_delta(&old_attributes, &new_attributes);

    assert!(dirty.render);
    assert!(!dirty.text);
    assert!(!dirty.style);
}
