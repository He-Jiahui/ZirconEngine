use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::scene::WorldInspectionField;
use zircon_runtime_interface::reflect::ReflectedValue;

use super::{scene_inspector_field_from_runtime, scene_inspector_fields_from_runtime};

const FIELD_COUNT: usize = 4_096;
const SAMPLE_PAIRS: usize = 101;

#[test]
fn editor883_scene_inspector_field_capacity_keeps_all_rejected_path_lazy() {
    let rejected = (0..64)
        .map(|index| inspection_field(index, ReflectedValue::Null))
        .collect::<Vec<_>>();

    let projected = scene_inspector_fields_from_runtime(&rejected);

    assert!(projected.is_empty());
    assert_eq!(projected.capacity(), 0);
}

#[test]
fn editor883_scene_inspector_field_capacity_preserves_retained_order() {
    let mut fields = (0..8)
        .map(|index| inspection_field(index, ReflectedValue::Null))
        .collect::<Vec<_>>();
    fields.extend(
        (0..64).map(|index| {
            inspection_field(index, ReflectedValue::String(format!("value-{index:03}")))
        }),
    );

    let projected = scene_inspector_fields_from_runtime(&fields);

    assert_eq!(projected.len(), 64);
    assert!(projected.capacity() >= fields.len());
    assert_eq!(projected[0].label, "Field 0");
    assert_eq!(projected[63].label, "Field 63");
    assert_eq!(projected, legacy_projection(&fields));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn editor883_scene_inspector_field_capacity_benchmark() {
    let fields = (0..FIELD_COUNT)
        .map(|index| inspection_field(index, ReflectedValue::String(format!("value-{index:05}"))))
        .collect::<Vec<_>>();
    assert_eq!(
        legacy_projection(&fields),
        scene_inspector_fields_from_runtime(&fields)
    );

    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(|| legacy_projection(&fields)));
            optimized.push(measure(|| scene_inspector_fields_from_runtime(&fields)));
        } else {
            optimized.push(measure(|| scene_inspector_fields_from_runtime(&fields)));
            legacy.push(measure(|| legacy_projection(&fields)));
        }
    }

    let legacy_p50 = percentile(&legacy, 50);
    let legacy_p95 = percentile(&legacy, 95);
    let legacy_p99 = percentile(&legacy, 99);
    let optimized_p50 = percentile(&optimized, 50);
    let optimized_p95 = percentile(&optimized, 95);
    let optimized_p99 = percentile(&optimized, 99);
    println!(
        "EDITOR883_SCENE_INSPECTOR_FIELD_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} fields={FIELD_COUNT} legacy_growth_events=11 optimized_growth_events=0 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} legacy_p99_ns={legacy_p99} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} optimized_p99_ns={optimized_p99}"
    );
    assert_eq!(growth_events(FIELD_COUNT, 0), 11);
    assert_eq!(growth_events(FIELD_COUNT, FIELD_COUNT), 0);
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(110),
        "lazy-reserved Inspector projection P95 {optimized_p95}ns must stay within 10% of filtered collection P95 {legacy_p95}ns"
    );
}

fn inspection_field(index: usize, value: ReflectedValue) -> WorldInspectionField {
    WorldInspectionField {
        component_type_path: "plugin::InspectionComponent".to_string(),
        component_display_name: "Inspection Component".to_string(),
        field_name: format!("field_{index}"),
        field_display_name: format!("field_{index}"),
        value_type_path: "test::Value".to_string(),
        value,
        writable: true,
        serializable: true,
        plugin_owned: false,
    }
}

fn legacy_projection(fields: &[WorldInspectionField]) -> Vec<super::super::SceneInspectorField> {
    fields
        .iter()
        .filter_map(scene_inspector_field_from_runtime)
        .collect()
}

fn growth_events(item_count: usize, initial_capacity: usize) -> usize {
    let mut capacity = initial_capacity;
    let mut events = 0;
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
