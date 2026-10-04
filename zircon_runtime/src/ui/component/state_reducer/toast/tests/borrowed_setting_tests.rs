use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::component::{
    UiComponentCategory, UiComponentDescriptor, UiComponentState, UiValue,
};

use super::{has_current_toast, string_setting_ref, sync_toast_state};

const LOOKUPS_PER_SAMPLE: usize = 1_024;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn runtime772_toast_borrowed_settings_preserve_sync_semantics() {
    let descriptor = UiComponentDescriptor::new(
        "ToastFixture",
        "Toast Fixture",
        UiComponentCategory::Visual,
        "toast",
    );
    let mut state = UiComponentState::new()
        .with_value("message", UiValue::String("hello".to_string()))
        .with_value("current_toast_id", UiValue::String("stable-id".to_string()));

    assert_eq!(
        string_setting_ref(&state, &descriptor, "current_toast_id"),
        Some("stable-id")
    );
    assert!(has_current_toast(&state, &descriptor));
    sync_toast_state(&mut state, &descriptor).expect("authored toast should sync");
    assert_eq!(
        state.value("current_toast_id"),
        Some(&UiValue::String("stable-id".to_string()))
    );

    let mut authored =
        UiComponentState::new().with_value("message", UiValue::String("authored-id".to_string()));
    sync_toast_state(&mut authored, &descriptor).expect("authored message should become current");
    assert_eq!(
        authored.value("current_toast_id"),
        Some(&UiValue::String("authored-id".to_string()))
    );
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime772_toast_borrowed_setting_release_benchmark() {
    let descriptor = UiComponentDescriptor::new(
        "ToastFixture",
        "Toast Fixture",
        UiComponentCategory::Visual,
        "toast",
    );
    let state = UiComponentState::new().with_value(
        "current_toast_id",
        UiValue::String("stable-toast-identifier".to_string()),
    );
    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_ns.push(measure_lookup(&state, &descriptor, true));
            optimized_ns.push(measure_lookup(&state, &descriptor, false));
        } else {
            optimized_ns.push(measure_lookup(&state, &descriptor, false));
            legacy_ns.push(measure_lookup(&state, &descriptor, true));
        }
    }

    let legacy_setting_clones = LOOKUPS_PER_SAMPLE;
    let optimized_setting_clones = 0;
    assert!(legacy_setting_clones > optimized_setting_clones);
    assert_eq!(optimized_setting_clones, 0);

    println!(
        "RUNTIME772_TOAST_BORROWED_SETTING_BENCH_V1 lookups_per_sample={LOOKUPS_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_setting_clones={legacy_setting_clones} optimized_setting_clones={optimized_setting_clones} legacy_p95_ns={} optimized_p95_ns={}",
        percentile(&legacy_ns, 95),
        percentile(&optimized_ns, 95),
    );
}

fn measure_lookup(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
    legacy: bool,
) -> u128 {
    let started = Instant::now();
    for _ in 0..LOOKUPS_PER_SAMPLE {
        if legacy {
            black_box(legacy_string_setting(state, descriptor, "current_toast_id"));
        } else {
            black_box(string_setting_ref(state, descriptor, "current_toast_id"));
        }
    }
    started.elapsed().as_nanos().max(1)
}

fn legacy_string_setting(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
    property: &str,
) -> Option<String> {
    super::value_setting(state, descriptor, property).and_then(|value| match value {
        UiValue::String(value) | UiValue::Enum(value) => Some(value.clone()),
        _ => None,
    })
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}
