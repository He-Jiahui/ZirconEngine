use std::hint::black_box;
use std::time::Instant;

use toml::Value;
use zircon_runtime_interface::ui::template::{UiChildMount, UiNodeDefinition};

use super::node_has_non_virtualized_scroll_child_pressure;

const PERF_MARKER: &str = "RUNTIME360_UI_INVALIDATION_SCROLL_CONTAINER_BENCH_V1";

fn scroll_node(virtualized: bool) -> UiNodeDefinition {
    let mut container = toml::map::Map::new();
    container.insert(
        "kind".to_string(),
        Value::String("ScrollableBox".to_string()),
    );
    if virtualized {
        container.insert("virtualization".to_string(), Value::Boolean(true));
    }
    let mut layout = std::collections::BTreeMap::new();
    layout.insert("container".to_string(), Value::Table(container));
    UiNodeDefinition {
        layout: Some(layout),
        children: (0..250)
            .map(|index| UiChildMount {
                node: UiNodeDefinition {
                    node_id: format!("child-{index}"),
                    ..UiNodeDefinition::default()
                },
                ..UiChildMount::default()
            })
            .collect(),
        ..UiNodeDefinition::default()
    }
}

#[test]
fn optimization_batch_20260830bh_runtime_scroll_pressure_preserves_virtualization_result() {
    assert!(node_has_non_virtualized_scroll_child_pressure(
        &scroll_node(false)
    ));
    assert!(!node_has_non_virtualized_scroll_child_pressure(
        &scroll_node(true)
    ));
    let widget_scroll = UiNodeDefinition {
        widget_type: Some("ScrollableBox".to_string()),
        children: (0..250)
            .map(|index| UiChildMount {
                node: UiNodeDefinition {
                    node_id: format!("child-{index}"),
                    ..UiNodeDefinition::default()
                },
                ..UiChildMount::default()
            })
            .collect(),
        ..UiNodeDefinition::default()
    };
    assert!(node_has_non_virtualized_scroll_child_pressure(
        &widget_scroll
    ));
}

#[test]
fn optimization_batch_20260830bh_runtime_scroll_pressure_source_contract() {
    let source = include_str!("../diagnostic.rs");
    assert!(source.contains("let container = node"));
    assert!(source.contains("container.is_some_and"));
    assert!(!source.contains("fn node_is_scrollable_box"));
    assert!(!source.contains("fn node_declares_virtualization"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260830bh_runtime_scroll_pressure_p95() {
    const NODES: usize = 2_000_000;
    const SAMPLES: usize = 17;
    let node = black_box(scroll_node(false));
    let mut baseline = Vec::with_capacity(SAMPLES);
    let mut candidate = Vec::with_capacity(SAMPLES);
    for sample in 0..SAMPLES {
        let order = if sample % 2 == 0 { [0, 1] } else { [1, 0] };
        for pass in order {
            let started = Instant::now();
            let mut checksum = 0usize;
            for _ in 0..NODES {
                let matched = if pass == 0 {
                    node.children.len() >= 250
                        && (node.widget_type.as_deref() == Some("ScrollableBox")
                            || node
                                .layout
                                .as_ref()
                                .and_then(|layout| layout.get("container"))
                                .and_then(Value::as_table)
                                .and_then(|container| container.get("kind"))
                                .and_then(Value::as_str)
                                == Some("ScrollableBox"))
                        && !node
                            .layout
                            .as_ref()
                            .and_then(|layout| layout.get("container"))
                            .and_then(Value::as_table)
                            .is_some_and(|container| container.contains_key("virtualization"))
                } else {
                    node_has_non_virtualized_scroll_child_pressure(&node)
                };
                checksum += usize::from(matched);
            }
            black_box(checksum);
            let elapsed = started.elapsed().as_nanos();
            if pass == 0 {
                baseline.push(elapsed);
            } else {
                candidate.push(elapsed);
            }
        }
    }
    baseline.sort_unstable();
    candidate.sort_unstable();
    let baseline_p95 = baseline[(SAMPLES * 95).div_ceil(100) - 1];
    let candidate_p95 = candidate[(SAMPLES * 95).div_ceil(100) - 1];
    let reduction =
        100.0 * baseline_p95.saturating_sub(candidate_p95) as f64 / baseline_p95.max(1) as f64;
    println!(
        "{PERF_MARKER} nodes={NODES} samples={SAMPLES} baseline_p95_ns={baseline_p95} candidate_p95_ns={candidate_p95} p95_reduction_percent={reduction:.2}"
    );
    assert!(candidate_p95.saturating_mul(10) <= baseline_p95.saturating_mul(7));
}
