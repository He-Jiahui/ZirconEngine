use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use serde_json::json;
use zircon_runtime_interface::ui::event_ui::UiTreeId;

use super::{
    activity_node, activity_property_capacity, EditorActivityHost, EditorActivityKind,
    EditorActivityReflection, SnapshotBuilder, ACTIVITY_CORE_PROPERTY_COUNT,
};

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 2_048;
const PROPERTIES_PER_BUILD: usize = 256;
const CUSTOM_PROPERTIES_PER_BUILD: usize = PROPERTIES_PER_BUILD - ACTIVITY_CORE_PROPERTY_COUNT;

#[test]
fn optimization_batch_20260826fc_editor144_capacity_preserves_activity_properties() {
    let custom_properties = (0..CUSTOM_PROPERTIES_PER_BUILD)
        .map(|index| (format!("custom-{index:03}"), json!(index)))
        .collect::<BTreeMap<_, _>>();
    let activity = EditorActivityReflection {
        instance_id: "editor144.activity".to_string(),
        descriptor_id: "editor144.descriptor".to_string(),
        title: "Editor 144".to_string(),
        kind: EditorActivityKind::ActivityView,
        host: EditorActivityHost::DocumentPage("workbench".to_string()),
        visible: true,
        enabled: true,
        dirty: false,
        properties: custom_properties,
        actions: Vec::new(),
    };
    let mut builder = SnapshotBuilder::new(UiTreeId::new("editor144.tree"));

    let node_id = activity_node(&mut builder, &activity, "activity/editor144".to_string());
    let snapshot = builder.finish(node_id);
    let node = snapshot.nodes.get(&node_id).expect("activity node");

    assert_eq!(activity_property_capacity(&activity), PROPERTIES_PER_BUILD);
    assert_eq!(node.properties.len(), PROPERTIES_PER_BUILD);
    assert!(node.properties.contains_key("descriptor_id"));
    assert!(node.properties.contains_key("host"));
    assert!(node.properties.contains_key("kind"));
    assert_eq!(node.properties["custom-252"].value, json!(252));
}

#[test]
fn optimization_batch_20260826fc_editor144_activity_properties_reserve_exact_count() {
    let source = include_str!("../activity.rs");
    assert!(source.contains("const ACTIVITY_CORE_PROPERTY_COUNT: usize = 3;"));
    assert!(source.contains("Vec::with_capacity(activity_property_capacity(activity))"));
    assert!(source.contains(".saturating_add(activity.properties.len())"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826fc_editor144_activity_property_capacity_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false));
            optimized_samples.push(measure(true));
        } else {
            optimized_samples.push(measure(true));
            legacy_samples.push(measure(false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR144_ACTIVITY_PROPERTY_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} core_properties={ACTIVITY_CORE_PROPERTY_COUNT} \
custom_properties={CUSTOM_PROPERTIES_PER_BUILD} properties_per_build={PROPERTIES_PER_BUILD} \
legacy_reservations_per_build=0 optimized_reservations_per_build=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let mut properties = if reserve {
            Vec::with_capacity(PROPERTIES_PER_BUILD)
        } else {
            Vec::new()
        };
        for property in 0..PROPERTIES_PER_BUILD {
            properties.push(black_box(property));
        }
        checksum ^= black_box(properties.len() ^ properties.capacity());
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
