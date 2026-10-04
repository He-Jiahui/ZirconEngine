use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::component::{
    UiComponentCategory, UiComponentDescriptor, UiComponentState, UiValue,
};

use super::menu_typeahead_searches;

fn descriptor() -> UiComponentDescriptor {
    UiComponentDescriptor::new(
        "MenuFixture",
        "Menu Fixture",
        UiComponentCategory::Visual,
        "menu",
    )
}

#[test]
fn runtime777_menu_typeahead_search_projection_preserves_buffer_and_fallback_order() {
    let descriptor = descriptor();
    let pasted = menu_typeahead_searches(&UiComponentState::new(), &descriptor, "ab")
        .expect("pasted searches");
    assert_eq!(pasted.len(), 1);
    assert_eq!(pasted[0].buffer, "ab");
    assert!(pasted[0].prefer_current);

    let active = UiComponentState::new()
        .with_value("typeahead_buffer", UiValue::String("b".to_string()))
        .with_value("typeahead_buffer_expired", UiValue::Bool(false));
    let searches = menu_typeahead_searches(&active, &descriptor, "a").expect("searches");
    assert_eq!(searches.len(), 2);
    assert_eq!(searches[0].buffer, "ba");
    assert!(searches[0].prefer_current);
    assert_eq!(searches[1].buffer, "a");
    assert!(!searches[1].prefer_current);

    let unicode = menu_typeahead_searches(&UiComponentState::new(), &descriptor, "\u{130}")
        .expect("Unicode searches");
    assert_eq!(unicode.len(), 1);
    assert_eq!(unicode[0].buffer, "i\u{307}");
    assert!(unicode[0].prefer_current);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime777_menu_typeahead_search_projection_release_benchmark() {
    const KEY_EVENTS_PER_SAMPLE: usize = 16_384;
    const SAMPLE_PAIRS: usize = 17;
    const PREVIOUS: &str = "editor-command";
    const PAYLOAD: &str = "p";

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_ns.push(measure_projection(
                false,
                KEY_EVENTS_PER_SAMPLE,
                PREVIOUS,
                PAYLOAD,
            ));
            optimized_ns.push(measure_projection(
                true,
                KEY_EVENTS_PER_SAMPLE,
                PREVIOUS,
                PAYLOAD,
            ));
        } else {
            optimized_ns.push(measure_projection(
                true,
                KEY_EVENTS_PER_SAMPLE,
                PREVIOUS,
                PAYLOAD,
            ));
            legacy_ns.push(measure_projection(
                false,
                KEY_EVENTS_PER_SAMPLE,
                PREVIOUS,
                PAYLOAD,
            ));
        }
    }

    let legacy_duplicate_search_string_allocations = KEY_EVENTS_PER_SAMPLE * 2;
    let optimized_duplicate_search_string_allocations = 0;
    assert!(
        legacy_duplicate_search_string_allocations > optimized_duplicate_search_string_allocations
    );
    println!(
        "RUNTIME777_MENU_TYPEAHEAD_SEARCH_PROJECTION_BENCH_V1 key_events_per_sample={KEY_EVENTS_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} legacy_duplicate_search_string_allocations={legacy_duplicate_search_string_allocations} optimized_duplicate_search_string_allocations={optimized_duplicate_search_string_allocations} legacy_p95_ns={} optimized_p95_ns={}",
        percentile(&legacy_ns, 95),
        percentile(&optimized_ns, 95),
    );
}

fn measure_projection(optimized: bool, key_events: usize, previous: &str, payload: &str) -> u128 {
    let started = Instant::now();
    let mut retained_bytes = 0usize;
    for _ in 0..key_events {
        let mut combined = String::with_capacity(previous.len() + payload.len());
        combined.push_str(previous);
        combined.push_str(black_box(payload));
        if optimized {
            retained_bytes += combined.len();
        } else {
            let search = combined.clone();
            retained_bytes += search.len() + combined.len();
        }
    }
    black_box(retained_bytes);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}
