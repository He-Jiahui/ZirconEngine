use std::hint::black_box;
use std::time::Instant;

use super::ViewTemplateNodePatch;
use zircon_runtime_interface::ui::component::UiValue;

const SAMPLE_PAIRS: usize = 31;
const CHECKS_PER_SAMPLE: usize = 200_000;

#[test]
fn optimization_batch_20260828ip_editor234_iterator_preserves_property_order_and_values() {
    let previous = patch(false, false, "surface.default", "text.primary");
    let desired = patch(true, true, "surface.raised", "text.warning");

    let changes = desired.changed_properties(&previous).collect::<Vec<_>>();

    assert_eq!(
        changes,
        vec![
            ("selected", UiValue::Bool(true)),
            ("focused", UiValue::Bool(true)),
            (
                "surface_variant",
                UiValue::String("surface.raised".to_string())
            ),
            ("text_tone", UiValue::String("text.warning".to_string())),
        ]
    );
}

#[test]
fn optimization_batch_20260828ip_editor234_patch_changes_use_fixed_slots() {
    let source = include_str!("../projection_patch.rs");
    let method_start = source
        .find("pub(super) fn changed_properties")
        .expect("changed_properties method");
    let method = &source[method_start..];

    assert!(method.contains("impl Iterator<Item = (&'static str, UiValue)"));
    assert!(method.contains(".into_iter()"));
    assert!(method.contains(".flatten()"));
    assert!(!method.contains("let mut properties = Vec::new()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260828ip_editor234_allocation_free_patch_changes_bench() {
    let previous = patch(false, false, "surface.default", "text.primary");
    let desired = patch(true, false, "surface.default", "text.primary");
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&desired, &previous, false));
            optimized_samples.push(measure(&desired, &previous, true));
        } else {
            optimized_samples.push(measure(&desired, &previous, true));
            legacy_samples.push(measure(&desired, &previous, false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR234_ALLOCATION_FREE_PATCH_CHANGES_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
checks_per_sample={CHECKS_PER_SAMPLE} changed_properties_per_check=1 \
legacy_heap_vectors_per_sample={CHECKS_PER_SAMPLE} optimized_heap_vectors_per_sample=0 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn patch(
    selected: bool,
    focused: bool,
    surface_variant: &str,
    text_tone: &str,
) -> ViewTemplateNodePatch {
    ViewTemplateNodePatch::visual_state(selected, focused, surface_variant, text_tone)
}

fn legacy_changed_properties(
    current: &ViewTemplateNodePatch,
    previous: &ViewTemplateNodePatch,
) -> Vec<(&'static str, UiValue)> {
    let mut properties = Vec::new();
    if current.selected != previous.selected {
        properties.push(("selected", UiValue::Bool(current.selected.unwrap_or(false))));
    }
    if current.focused != previous.focused {
        properties.push(("focused", UiValue::Bool(current.focused.unwrap_or(false))));
    }
    if current.surface_variant != previous.surface_variant {
        properties.push((
            "surface_variant",
            UiValue::String(current.surface_variant.clone().unwrap_or_default()),
        ));
    }
    if current.text_tone != previous.text_tone {
        properties.push((
            "text_tone",
            UiValue::String(current.text_tone.clone().unwrap_or_default()),
        ));
    }
    properties
}

fn measure(
    current: &ViewTemplateNodePatch,
    previous: &ViewTemplateNodePatch,
    optimized: bool,
) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..CHECKS_PER_SAMPLE {
        if optimized {
            for (name, value) in black_box(current.changed_properties(black_box(previous))) {
                checksum ^= black_box(name.len());
                black_box(value);
            }
        } else {
            for (name, value) in black_box(legacy_changed_properties(
                black_box(current),
                black_box(previous),
            )) {
                checksum ^= black_box(name.len());
                black_box(value);
            }
        }
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
