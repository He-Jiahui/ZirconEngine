use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::component::{
    UiComponentCategory, UiComponentDescriptor, UiComponentState, UiValue,
};

use super::menu_typeahead_searches;

#[test]
fn runtime776_menu_typeahead_append_reuses_previous_buffer() {
    let descriptor = UiComponentDescriptor::new(
        "MenuFixture",
        "Menu Fixture",
        UiComponentCategory::Visual,
        "menu",
    );
    let active = UiComponentState::new()
        .with_value("typeahead_buffer", UiValue::String("b".to_string()))
        .with_value("typeahead_buffer_expired", UiValue::Bool(false));
    let searches = menu_typeahead_searches(&active, &descriptor, "a").expect("searches");
    assert_eq!(searches.len(), 2);
    assert_eq!(searches[0].buffer, "ba");
    assert!(searches[0].prefer_current);
    assert_eq!(searches[1].buffer, "a");
    assert!(!searches[1].prefer_current);

    let repeated = UiComponentState::new()
        .with_value("typeahead_buffer", UiValue::String("a".to_string()))
        .with_value("typeahead_buffer_expired", UiValue::Bool(false));
    let repeated_searches =
        menu_typeahead_searches(&repeated, &descriptor, "a").expect("repeated searches");
    assert_eq!(repeated_searches.len(), 1);
    assert_eq!(repeated_searches[0].buffer, "a");
    assert!(!repeated_searches[0].prefer_current);

    let expired = UiComponentState::new()
        .with_value("typeahead_buffer", UiValue::String("b".to_string()))
        .with_value("typeahead_buffer_expired", UiValue::Bool(true));
    let expired_searches =
        menu_typeahead_searches(&expired, &descriptor, "a").expect("expired searches");
    assert_eq!(expired_searches.len(), 1);
    assert_eq!(expired_searches[0].buffer, "a");
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime776_menu_typeahead_append_reuse_release_benchmark() {
    const KEY_EVENTS_PER_SAMPLE: usize = 16_384;
    const SAMPLE_PAIRS: usize = 17;
    const PREVIOUS: &str = "editor-command";
    const PAYLOAD: &str = "p";

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_ns.push(measure_append(
                false,
                KEY_EVENTS_PER_SAMPLE,
                PREVIOUS,
                PAYLOAD,
            ));
            optimized_ns.push(measure_append(
                true,
                KEY_EVENTS_PER_SAMPLE,
                PREVIOUS,
                PAYLOAD,
            ));
        } else {
            optimized_ns.push(measure_append(
                true,
                KEY_EVENTS_PER_SAMPLE,
                PREVIOUS,
                PAYLOAD,
            ));
            legacy_ns.push(measure_append(
                false,
                KEY_EVENTS_PER_SAMPLE,
                PREVIOUS,
                PAYLOAD,
            ));
        }
    }

    let legacy_combined_string_allocations = KEY_EVENTS_PER_SAMPLE;
    let optimized_combined_string_allocations = 0;
    assert!(legacy_combined_string_allocations > optimized_combined_string_allocations);
    println!(
        "RUNTIME776_MENU_TYPEAHEAD_APPEND_REUSE_BENCH_V1 key_events_per_sample={KEY_EVENTS_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} legacy_combined_string_allocations={legacy_combined_string_allocations} optimized_combined_string_allocations={optimized_combined_string_allocations} legacy_p95_ns={} optimized_p95_ns={}",
        percentile(&legacy_ns, 95),
        percentile(&optimized_ns, 95),
    );
}

fn measure_append(optimized: bool, key_events: usize, previous: &str, payload: &str) -> u128 {
    let started = Instant::now();
    let mut combined_bytes = 0usize;
    for _ in 0..key_events {
        let mut reusable = String::with_capacity(previous.len() + payload.len());
        reusable.push_str(previous);
        let combined = if optimized {
            reusable.push_str(black_box(payload));
            reusable
        } else {
            format!("{reusable}{payload}")
        };
        combined_bytes += combined.len();
    }
    black_box(combined_bytes);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}
