use std::hint::black_box;
use std::time::Instant;

use crate::ui::component::UiComponentDescriptorRegistry;
use zircon_runtime_interface::ui::component::{UiComponentDescriptor, UiComponentState, UiValue};

use super::{is_confirm_action, string_setting_ref};

const SAMPLE_PAIRS: usize = 21;
const LOOKUPS_PER_SAMPLE: usize = 131_072;

#[test]
fn optimization_batch_20260826ec_runtime172_confirm_action_preserves_value_and_default_semantics() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let descriptor = registry
        .descriptor("ConfirmDialog")
        .expect("ConfirmDialog descriptor");
    let state = UiComponentState::new().with_value(
        "confirm_action_id",
        UiValue::String("delete-selected-node".to_string()),
    );

    assert!(is_confirm_action(
        &state,
        descriptor,
        "delete-selected-node"
    ));
    assert!(is_confirm_action(&state, descriptor, "confirm"));
    assert!(!is_confirm_action(&state, descriptor, "cancel"));
    assert_eq!(
        string_setting_ref(&UiComponentState::new(), descriptor, "confirm_action_id"),
        Some("confirm")
    );
}

#[test]
fn optimization_batch_20260826ec_runtime172_confirm_action_borrows_state_value() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let descriptor = registry
        .descriptor("ConfirmDialog")
        .expect("ConfirmDialog descriptor");
    let state = UiComponentState::new().with_value(
        "confirm_action_id",
        UiValue::String("publish-current-selection".to_string()),
    );
    let stored = match state.value("confirm_action_id") {
        Some(UiValue::String(value)) => value.as_str(),
        value => panic!("expected stored string, got {value:?}"),
    };
    let borrowed = string_setting_ref(&state, descriptor, "confirm_action_id").unwrap();

    assert_eq!(borrowed.as_ptr(), stored.as_ptr());
    let source = include_str!("../overlay.rs");
    let helper_start = source.find("fn string_setting_ref").unwrap();
    let helper_end = source[helper_start..]
        .find("fn string_value_ref")
        .map(|offset| helper_start + offset)
        .unwrap();
    assert!(!source[helper_start..helper_end].contains("clone()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826ec_runtime172_overlay_borrowed_confirm_action_bench() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let descriptor = registry
        .descriptor("ConfirmDialog")
        .expect("ConfirmDialog descriptor");
    let action_id = "publish-current-selection-with-a-stable-operation-identifier";
    let state = UiComponentState::new()
        .with_value("confirm_action_id", UiValue::String(action_id.to_string()));
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy(&state, descriptor, action_id));
            optimized_samples.push(measure_optimized(&state, descriptor, action_id));
        } else {
            optimized_samples.push(measure_optimized(&state, descriptor, action_id));
            legacy_samples.push(measure_legacy(&state, descriptor, action_id));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME172_OVERLAY_BORROWED_CONFIRM_ACTION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
lookups_per_sample={LOOKUPS_PER_SAMPLE} legacy_allocations_per_lookup=1 \
optimized_allocations_per_lookup=0 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "borrowed confirm-action comparison P95 {optimized_p95_ns}ns must be at most 70% of cloned comparison P95 {legacy_p95_ns}ns"
    );
}

fn legacy_string_setting(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
    property: &str,
) -> Option<String> {
    state
        .values
        .get(property)
        .and_then(legacy_string_value)
        .or_else(|| {
            descriptor
                .prop(property)
                .and_then(|schema| schema.default_value.as_ref())
                .and_then(legacy_string_value)
        })
}

fn legacy_string_value(value: &UiValue) -> Option<String> {
    match value {
        UiValue::String(value) | UiValue::Enum(value) => Some(value.clone()),
        _ => None,
    }
}

fn legacy_is_confirm_action(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
    action_id: &str,
) -> bool {
    let confirm_action = legacy_string_setting(state, descriptor, "confirm_action_id")
        .unwrap_or_else(|| "confirm".to_string());
    action_id == confirm_action || action_id == "confirm"
}

fn measure_legacy(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
    action_id: &str,
) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..LOOKUPS_PER_SAMPLE {
        checksum ^= black_box(legacy_is_confirm_action(
            black_box(state),
            black_box(descriptor),
            black_box(action_id),
        )) as usize;
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
    action_id: &str,
) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..LOOKUPS_PER_SAMPLE {
        checksum ^= black_box(is_confirm_action(
            black_box(state),
            black_box(descriptor),
            black_box(action_id),
        )) as usize;
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
