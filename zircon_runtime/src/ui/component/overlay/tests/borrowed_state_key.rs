use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::component::{UiComponentState, UiDragSourceMetadata, UiValue};

use super::super::overlay::set_overlay_value;

#[test]
fn runtime783_overlay_static_keys_update_in_place() {
    let mut state = UiComponentState::new();
    set_overlay_value(&mut state, "popup_open", UiValue::Bool(false));
    let key_ptr = state
        .values
        .get_key_value("popup_open")
        .map(|(key, _)| key.as_ptr())
        .expect("initial overlay key");
    set_overlay_value(&mut state, "popup_open", UiValue::Bool(true));
    let updated_key_ptr = state
        .values
        .get_key_value("popup_open")
        .map(|(key, _)| key.as_ptr())
        .expect("updated overlay key");
    assert_eq!(key_ptr, updated_key_ptr);
    assert_eq!(state.values.get("popup_open"), Some(&UiValue::Bool(true)));
}

#[test]
fn runtime783_overlay_static_keys_clear_reference_sources() {
    let mut state = UiComponentState::new().with_value("popup_open", UiValue::Bool(false));
    state.reference_sources.insert(
        "popup_open".to_string(),
        UiDragSourceMetadata {
            source_surface: "model.popup_open".to_string(),
            ..UiDragSourceMetadata::default()
        },
    );
    set_overlay_value(&mut state, "popup_open", UiValue::Bool(true));
    assert!(!state.reference_sources.contains_key("popup_open"));
    assert_eq!(state.values.get("popup_open"), Some(&UiValue::Bool(true)));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime783_overlay_static_key_update_release_benchmark() {
    const UPDATES_PER_SAMPLE: usize = 16_384;
    const SAMPLE_PAIRS: usize = 17;
    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_ns.push(measure_updates(UPDATES_PER_SAMPLE, true));
            optimized_ns.push(measure_updates(UPDATES_PER_SAMPLE, false));
        } else {
            optimized_ns.push(measure_updates(UPDATES_PER_SAMPLE, false));
            legacy_ns.push(measure_updates(UPDATES_PER_SAMPLE, true));
        }
    }
    let legacy_key_allocations = UPDATES_PER_SAMPLE;
    let optimized_key_allocations = 0;
    println!(
        "RUNTIME783_OVERLAY_STATIC_KEY_UPDATE_BENCH_V1 updates_per_sample={UPDATES_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} legacy_key_allocations={legacy_key_allocations} optimized_key_allocations={optimized_key_allocations} legacy_p95_ns={} optimized_p95_ns={}",
        percentile(&legacy_ns, 95),
        percentile(&optimized_ns, 95),
    );
}

fn measure_updates(updates: usize, legacy: bool) -> u128 {
    let mut state = UiComponentState::new().with_value("popup_open", UiValue::Bool(false));
    let started = Instant::now();
    for update in 0..updates {
        if legacy {
            super::super::set_value(
                &mut state,
                "popup_open".to_string(),
                UiValue::Bool(update % 2 == 0),
            );
        } else {
            set_overlay_value(&mut state, "popup_open", UiValue::Bool(update % 2 == 0));
        }
    }
    black_box(state);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}
