use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::{Duration, Instant};

use toml::Value;

use crate::ui::template_runtime::RetainedUiHostNodeProjection;

use super::merge_projection_metadata;

const SAMPLE_PAIRS: usize = 21;
const ITERATIONS_PER_SAMPLE: usize = 32;
const ENTRIES_PER_MAP: usize = 512;

#[test]
fn optimization_batch_ho_editor596_owned_projection_metadata_preserves_live_attributes_and_styles()
{
    let mut surface = host_node("surface");
    surface
        .attributes
        .insert("state".to_string(), Value::String("old".to_string()));
    surface
        .style_tokens
        .insert("accent".to_string(), "old".to_string());
    surface
        .style_overrides
        .insert("opacity".to_string(), Value::Float(0.5));

    let mut projection = host_node("projection");
    projection
        .attributes
        .insert("state".to_string(), Value::String("new".to_string()));
    projection
        .style_tokens
        .insert("accent".to_string(), "new".to_string());
    projection
        .style_overrides
        .insert("opacity".to_string(), Value::Float(1.0));

    surface
        .attributes
        .insert("value".to_string(), Value::String(String::new()));
    surface
        .attributes
        .insert("selected".to_string(), Value::Boolean(false));
    projection
        .attributes
        .insert("value".to_string(), Value::String("128.4".to_string()));
    projection
        .attributes
        .insert("selected".to_string(), Value::Boolean(true));
    projection
        .attributes
        .insert("source_only".to_string(), Value::String("kept".to_string()));
    merge_projection_metadata(&mut surface, projection);
    assert_eq!(surface.attributes["value"].as_str(), Some(""));
    assert_eq!(surface.attributes["selected"].as_bool(), Some(false));
    assert_eq!(surface.attributes["source_only"].as_str(), Some("kept"));

    assert_eq!(surface.attributes["state"].as_str(), Some("old"));
    assert_eq!(surface.style_tokens["accent"], "new");
    assert_eq!(surface.style_overrides["opacity"].as_float(), Some(1.0));
}

#[test]
fn optimization_batch_ho_editor596_matched_projection_metadata_is_moved_into_surface_nodes() {
    let source = include_str!("../../projection.rs");
    let merge = source
        .split("fn merge_projection_metadata(")
        .nth(1)
        .expect("metadata merge")
        .split("fn project_node(")
        .next()
        .expect("metadata merge body");

    assert!(merge.contains("projection_node: RetainedUiHostNodeProjection"));
    assert_eq!(merge.matches(".append(").count(), 2);
    assert!(merge.contains("surface_node.attributes.entry(key).or_insert(value)"));
    assert!(!merge.contains("key.clone()"));
    assert!(!merge.contains("value.clone()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_ho_editor596_projection_metadata_move_p95() {
    const MARKER: &str = "EDITOR596_PROJECTION_METADATA_MOVE_BENCH_V1";
    let projection = large_projection_node();
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(&projection, false));
            optimized.push(measure(&projection, true));
        } else {
            optimized.push(measure(&projection, true));
            legacy.push(measure(&projection, false));
        }
    }

    let legacy_p95_ns = percentile_ns(&legacy, 95);
    let optimized_p95_ns = percentile_ns(&optimized, 95);
    let ratio = optimized_p95_ns as f64 / legacy_p95_ns.max(1) as f64;
    eprintln!(
        "{MARKER} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} ratio={ratio:.4}"
    );
    assert!(
        ratio <= 0.60,
        "{MARKER} expected ownership merge ratio <= 0.60, got {ratio:.4}"
    );
}

fn measure(projection: &RetainedUiHostNodeProjection, optimized: bool) -> Duration {
    let mut elapsed = Duration::ZERO;
    for _ in 0..ITERATIONS_PER_SAMPLE {
        let input = projection.clone();
        let mut target = host_node("surface");
        let start = Instant::now();
        if optimized {
            optimized_merge_model(&mut target, input);
        } else {
            legacy_merge_model(&mut target, &input);
        }
        black_box(&target);
        elapsed += start.elapsed();
    }
    elapsed
}

fn legacy_merge_model(
    target: &mut RetainedUiHostNodeProjection,
    source: &RetainedUiHostNodeProjection,
) {
    for (key, value) in &source.attributes {
        target.attributes.insert(key.clone(), value.clone());
    }
    for (key, value) in &source.style_tokens {
        target.style_tokens.insert(key.clone(), value.clone());
    }
    for (key, value) in &source.style_overrides {
        target.style_overrides.insert(key.clone(), value.clone());
    }
}

fn optimized_merge_model(
    target: &mut RetainedUiHostNodeProjection,
    source: RetainedUiHostNodeProjection,
) {
    let RetainedUiHostNodeProjection {
        mut attributes,
        mut style_overrides,
        mut style_tokens,
        ..
    } = source;
    target.attributes.append(&mut attributes);
    target.style_tokens.append(&mut style_tokens);
    target.style_overrides.append(&mut style_overrides);
}

fn large_projection_node() -> RetainedUiHostNodeProjection {
    let mut node = host_node("projection");
    for index in 0..ENTRIES_PER_MAP {
        node.attributes.insert(
            format!("attribute-{index:04}"),
            Value::String(format!("value-{index:04}")),
        );
        node.style_tokens.insert(
            format!("token-{index:04}"),
            format!("palette.value.{index:04}"),
        );
        node.style_overrides
            .insert(format!("override-{index:04}"), Value::Integer(index as i64));
    }
    node
}

fn host_node(node_id: &str) -> RetainedUiHostNodeProjection {
    RetainedUiHostNodeProjection {
        node_id: node_id.to_string(),
        surface_node_id: None,
        has_workbench_icon_tooltip: false,
        parent_id: None,
        component: "Panel".to_string(),
        control_id: Some(node_id.to_string()),
        source_path: None,
        source_node_id: None,
        instance_path: None,
        parent_source_path: None,
        parent_source_node_id: None,
        parent_instance_path: None,
        frame: Default::default(),
        clip_frame: None,
        z_index: 0,
        attributes: BTreeMap::new(),
        style_overrides: BTreeMap::new(),
        style_tokens: BTreeMap::new(),
        bindings: Vec::new(),
    }
}

fn percentile_ns(samples: &[Duration], percentile: usize) -> u128 {
    let mut values = samples.iter().map(Duration::as_nanos).collect::<Vec<_>>();
    values.sort_unstable();
    let index = (values.len() - 1) * percentile / 100;
    values[index]
}
