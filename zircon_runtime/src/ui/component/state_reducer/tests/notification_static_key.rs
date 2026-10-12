use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::component::{UiComponentState, UiDragSourceMetadata, UiValue};

use super::set_notification_value;

#[test]
fn runtime784_notification_static_keys_update_in_place() {
    let mut state = UiComponentState::new();
    set_notification_value(&mut state, "focused_index", UiValue::Int(-1));
    let key_ptr = state
        .values
        .get_key_value("focused_index")
        .map(|(key, _)| key.as_ptr())
        .expect("initial notification key");
    set_notification_value(&mut state, "focused_index", UiValue::Int(4));
    let updated_key_ptr = state
        .values
        .get_key_value("focused_index")
        .map(|(key, _)| key.as_ptr())
        .expect("updated notification key");
    assert_eq!(key_ptr, updated_key_ptr);
    assert_eq!(state.values.get("focused_index"), Some(&UiValue::Int(4)));
}

#[test]
fn runtime784_notification_static_keys_clear_reference_sources() {
    let mut state = UiComponentState::new().with_value(
        "selected_notification_id",
        UiValue::String("first".to_owned()),
    );
    state.reference_sources.insert(
        "selected_notification_id".to_owned(),
        UiDragSourceMetadata {
            source_surface: "model.selected_notification_id".to_owned(),
            ..UiDragSourceMetadata::default()
        },
    );
    set_notification_value(
        &mut state,
        "selected_notification_id",
        UiValue::String("second".to_owned()),
    );
    assert!(!state
        .reference_sources
        .contains_key("selected_notification_id"));
    assert_eq!(
        state.values.get("selected_notification_id"),
        Some(&UiValue::String("second".to_owned()))
    );
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime784_notification_static_key_update_release_benchmark() {
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
    let legacy_key_allocations = UPDATES_PER_SAMPLE * 3;
    let optimized_key_allocations = 0;
    println!(
        "RUNTIME784_NOTIFICATION_STATIC_KEY_UPDATE_BENCH_V1 updates_per_sample={UPDATES_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} legacy_key_allocations={legacy_key_allocations} optimized_key_allocations={optimized_key_allocations} legacy_p95_ns={} optimized_p95_ns={}",
        percentile(&legacy_ns, 95),
        percentile(&optimized_ns, 95),
    );
}

fn measure_updates(updates: usize, legacy: bool) -> u128 {
    let mut state = UiComponentState::new()
        .with_value(
            "selected_notification_id",
            UiValue::String("first".to_owned()),
        )
        .with_value("focused_index", UiValue::Int(-1))
        .with_value("unread_count", UiValue::Int(0));
    let started = Instant::now();
    for update in 0..updates {
        let selected = if update % 2 == 0 { "first" } else { "second" };
        if legacy {
            super::super::set_value(
                &mut state,
                "selected_notification_id".to_owned(),
                UiValue::String(selected.to_owned()),
            );
            super::super::set_value(
                &mut state,
                "focused_index".to_owned(),
                UiValue::Int(update as i64),
            );
            super::super::set_value(
                &mut state,
                "unread_count".to_owned(),
                UiValue::Int((update & 1) as i64),
            );
        } else {
            set_notification_value(
                &mut state,
                "selected_notification_id",
                UiValue::String(selected.to_owned()),
            );
            set_notification_value(&mut state, "focused_index", UiValue::Int(update as i64));
            set_notification_value(
                &mut state,
                "unread_count",
                UiValue::Int((update & 1) as i64),
            );
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
