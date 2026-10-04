use std::collections::BTreeMap;

use toml::Value;
use zircon_runtime_interface::ui::component::UiComponentState;
use zircon_runtime_interface::ui::event_ui::{UiNodeId, UiNodePath};
use zircon_runtime_interface::ui::tree::{UiTemplateNodeMetadata, UiTreeNode};

use super::{collect_runtime_pseudo_states, runtime_pseudo_state_initial_capacity};

#[test]
fn runtime_pseudo_state_capacity_preserves_order() {
    let mut node = UiTreeNode::new(UiNodeId(1), UiNodePath::new("root/button"));
    node.template_metadata = Some(UiTemplateNodeMetadata {
        attributes: BTreeMap::from([
            ("custom_b".to_owned(), Value::Boolean(true)),
            ("custom_a".to_owned(), Value::Boolean(true)),
        ]),
        ..UiTemplateNodeMetadata::default()
    });
    let mut component_state = UiComponentState::new();
    component_state.flags.hovered = true;
    component_state.flags.focus_visible = true;

    let states = collect_runtime_pseudo_states(&node, Some(&component_state));

    assert!(states.windows(2).all(|pair| pair[0] <= pair[1]));
    assert!(states.contains(&"custom_a".to_owned()));
    assert!(states.contains(&"resolved-hovered".to_owned()));
    assert!(
        states.capacity() >= runtime_pseudo_state_initial_capacity(&node, Some(&component_state),)
    );
}

#[test]
fn runtime_pseudo_state_capacity_keeps_clean_nodes_small() {
    let node = UiTreeNode::new(UiNodeId(2), UiNodePath::new("root/label"));

    let states = collect_runtime_pseudo_states(&node, None);

    assert_eq!(states, vec!["resolved-normal".to_owned()]);
    assert_eq!(runtime_pseudo_state_initial_capacity(&node, None), 2);
    assert!(states.capacity() >= 2);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime841_runtime_pseudo_state_capacity_release_benchmark() {
    const AUTHORED_STATE_COUNT: usize = 4_096;
    const BUILDS_PER_SAMPLE: usize = 4_096;
    const SAMPLE_PAIRS: usize = 17;
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_growth(
                AUTHORED_STATE_COUNT,
                BUILDS_PER_SAMPLE,
                false,
            ));
            optimized_samples.push(measure_growth(
                AUTHORED_STATE_COUNT,
                BUILDS_PER_SAMPLE,
                true,
            ));
        } else {
            optimized_samples.push(measure_growth(
                AUTHORED_STATE_COUNT,
                BUILDS_PER_SAMPLE,
                true,
            ));
            legacy_samples.push(measure_growth(
                AUTHORED_STATE_COUNT,
                BUILDS_PER_SAMPLE,
                false,
            ));
        }
    }
    assert!(growth_events(AUTHORED_STATE_COUNT + 26) > 0);
    assert_eq!(growth_events(AUTHORED_STATE_COUNT + 26), 12);
    println!(
        "RUNTIME841_RUNTIME_PSEUDO_STATE_CAPACITY_BENCH_V1 authored_state_count={AUTHORED_STATE_COUNT} builds_per_sample={BUILDS_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} legacy_p95_ns={} optimized_p95_ns={} legacy_growth_events={} optimized_growth_events=0",
        percentile(&legacy_samples, 95),
        percentile(&optimized_samples, 95),
        growth_events(AUTHORED_STATE_COUNT * 2 + 26),
    );
}

fn measure_growth(state_count: usize, builds: usize, optimized: bool) -> u128 {
    let started = std::time::Instant::now();
    let mut checksum = 0usize;
    for _ in 0..builds {
        let mut values = if optimized {
            Vec::with_capacity(state_count + 26)
        } else {
            Vec::new()
        };
        for index in 0..state_count {
            values.push(format!("custom-{index}"));
            values.push(format!("alias-{index}"));
        }
        checksum = checksum.wrapping_add(values.len());
        std::hint::black_box(values);
    }
    std::hint::black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn growth_events(item_count: usize) -> usize {
    let mut capacity = 0usize;
    let mut events = 0usize;
    for length in 1..=item_count {
        if length > capacity {
            capacity = if capacity == 0 { 4 } else { capacity * 2 };
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
