use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::{Duration, Instant};

use toml::Value;

use super::{RetainedUiNodeProjection, RetainedUiProjection};

const SAMPLE_COUNT: usize = 17;
const ITERATIONS: usize = 256;
const NODE_COUNT: usize = 1_024;
const ATTRIBUTE_COUNT: usize = 8;

fn percentile_95(mut samples: Vec<Duration>) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() * 95).div_ceil(100) - 1]
}

fn fixture_projection() -> RetainedUiProjection {
    RetainedUiProjection {
        document_id: "optimization.fixture".to_string(),
        bindings: Vec::new(),
        root: RetainedUiNodeProjection {
            component: "Root".to_string(),
            control_id: None,
            attributes: BTreeMap::new(),
            style_tokens: BTreeMap::new(),
            binding_ids: Vec::new(),
            children: (0..NODE_COUNT)
                .map(|node_index| RetainedUiNodeProjection {
                    component: "Button".to_string(),
                    control_id: Some(format!("Control{node_index:04}")),
                    attributes: (0..ATTRIBUTE_COUNT)
                        .map(|attribute_index| {
                            (
                                format!("attribute-{attribute_index}"),
                                Value::String(format!("value-{node_index}-{attribute_index}")),
                            )
                        })
                        .collect(),
                    style_tokens: (0..ATTRIBUTE_COUNT)
                        .map(|token_index| {
                            (
                                format!("token-{token_index}"),
                                format!("value-{node_index}-{token_index}"),
                            )
                        })
                        .collect(),
                    binding_ids: Vec::new(),
                    children: Vec::new(),
                })
                .collect(),
        },
    }
}

fn measure_samples(mut operation: impl FnMut()) -> Vec<Duration> {
    (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            operation();
            started.elapsed()
        })
        .collect()
}

fn legacy_metadata_index(projection: &RetainedUiProjection) -> usize {
    let mut by_control_id =
        BTreeMap::<String, (BTreeMap<String, Value>, BTreeMap<String, String>)>::new();
    let mut stack = vec![&projection.root];
    while let Some(node) = stack.pop() {
        if let Some(control_id) = node.control_id.as_ref() {
            let metadata = by_control_id.entry(control_id.clone()).or_default();
            metadata.0.extend(node.attributes.clone());
            metadata.1.extend(node.style_tokens.clone());
        }
        stack.extend(node.children.iter().rev());
    }
    by_control_id.len()
}

#[test]
fn editor01_surface_metadata_merges_preserve_preorder_and_patch_values() {
    let projection = fixture_projection();
    let index = projection.surface_metadata_index();
    let (attributes, style_tokens) = index.metadata_for("Control0000").unwrap();
    assert_eq!(attributes.len(), ATTRIBUTE_COUNT);
    assert_eq!(style_tokens.len(), ATTRIBUTE_COUNT);

    let mut target_attributes =
        BTreeMap::from([("attribute-0".to_string(), Value::String("old".to_string()))]);
    let mut target_style_tokens = BTreeMap::from([("token-0".to_string(), "old".to_string())]);
    index.apply_to(
        Some("Control0000"),
        &mut target_attributes,
        &mut target_style_tokens,
    );
    assert_eq!(target_attributes.len(), ATTRIBUTE_COUNT);
    assert_eq!(target_style_tokens.len(), ATTRIBUTE_COUNT);
    assert_eq!(
        target_attributes.get("attribute-0"),
        Some(&Value::String("value-0-0".to_string()))
    );
    assert_eq!(
        target_style_tokens.get("token-0"),
        Some(&"value-0-0".to_string())
    );
}

#[test]
fn editor01_surface_metadata_merge_avoids_temporary_map_clones() {
    let source = include_str!("../model.rs");
    let production = source
        .split_once("#[cfg(test)]")
        .expect("tests follow production")
        .0;
    assert!(!production.contains("metadata.attributes.clone()"));
    assert!(!production.contains("metadata.style_tokens.clone()"));
    assert!(production.contains("metadata\n                .attributes\n                .iter()"));
    assert!(production.contains("metadata\n                .style_tokens\n                .iter()"));
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn editor01_single_pass_surface_metadata_index_merge_bench() {
    let projection = fixture_projection();
    let legacy = measure_samples(|| {
        for _ in 0..ITERATIONS {
            black_box(legacy_metadata_index(&projection));
        }
    });
    let optimized = measure_samples(|| {
        for _ in 0..ITERATIONS {
            black_box(projection.surface_metadata_index());
        }
    });
    let legacy_p95 = percentile_95(legacy);
    let optimized_p95 = percentile_95(optimized);

    println!(
        "EDITOR01_SINGLE_PASS_SURFACE_METADATA_INDEX_MERGE_BENCH_V1 legacy_p95_ns={} optimized_p95_ns={} samples={} iterations={} nodes={} attributes_per_node={} temporary_attribute_maps=2,048->0 temporary_style_maps=2,048->0",
        legacy_p95.as_nanos(),
        optimized_p95.as_nanos(),
        SAMPLE_COUNT,
        ITERATIONS,
        NODE_COUNT,
        ATTRIBUTE_COUNT,
    );
    assert!(
        optimized_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 85,
        "optimized p95 should be at most 85% of legacy p95"
    );
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn editor01_single_pass_surface_metadata_apply_merge_bench() {
    let projection = fixture_projection();
    let index = projection.surface_metadata_index();
    let legacy = measure_samples(|| {
        for _ in 0..ITERATIONS {
            let (attributes, style_tokens) = index.metadata_for("Control0000").unwrap();
            let mut target_attributes = BTreeMap::new();
            let mut target_style_tokens = BTreeMap::new();
            target_attributes.extend(attributes.clone());
            target_style_tokens.extend(style_tokens.clone());
            black_box((target_attributes, target_style_tokens));
        }
    });
    let optimized = measure_samples(|| {
        for _ in 0..ITERATIONS {
            let mut target_attributes = BTreeMap::new();
            let mut target_style_tokens = BTreeMap::new();
            index.apply_to(
                Some("Control0000"),
                &mut target_attributes,
                &mut target_style_tokens,
            );
            black_box((target_attributes, target_style_tokens));
        }
    });
    let legacy_p95 = percentile_95(legacy);
    let optimized_p95 = percentile_95(optimized);

    println!(
        "EDITOR01_SINGLE_PASS_SURFACE_METADATA_APPLY_MERGE_BENCH_V1 legacy_p95_ns={} optimized_p95_ns={} samples={} iterations={} attributes={} style_tokens={} temporary_attribute_maps=1->0 temporary_style_maps=1->0",
        legacy_p95.as_nanos(),
        optimized_p95.as_nanos(),
        SAMPLE_COUNT,
        ITERATIONS,
        ATTRIBUTE_COUNT,
        ATTRIBUTE_COUNT,
    );
    assert!(
        optimized_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 85,
        "optimized p95 should be at most 85% of legacy p95"
    );
}
