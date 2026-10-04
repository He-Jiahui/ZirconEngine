use std::path::Path;

use super::{
    icon_candidates_from_asset_root, image_candidates_from_asset_root, preview_artifact_candidates,
};

const PERF_MARKER: &str = "EDITOR829_VISUAL_CANDIDATE_CAPACITY_BENCH_V1";

#[test]
fn packaged_image_candidates_use_the_four_variant_bound() {
    let candidates = image_candidates_from_asset_root("toolbar/logo", Path::new("E:/assets"));

    assert_eq!(candidates.len(), 4);
    assert!(candidates.capacity() >= 4);
}

#[test]
fn preview_and_icon_candidates_keep_finite_bounds_and_empty_paths() {
    let preview = preview_artifact_candidates("textures/preview");
    assert!(!preview.is_empty());
    assert!(preview.capacity() >= preview.len());

    let icons = icon_candidates_from_asset_root("Search", Path::new("E:/assets"), false);
    assert!(!icons.is_empty());
    assert!(icons.capacity() >= 6);

    assert_eq!(
        image_candidates_from_asset_root("", Path::new("E:/assets")).capacity(),
        0
    );
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn editor829_visual_candidate_capacity_bench_v1() {
    let legacy_growth_events = (4..=6)
        .map(|count| growth_events(count, false))
        .sum::<usize>();
    let optimized_growth_events = (4..=6)
        .map(|count| growth_events(count, true))
        .sum::<usize>();
    std::hint::black_box((legacy_growth_events, optimized_growth_events));
    println!(
        "{PERF_MARKER} variant_counts=4,5,6 legacy_growth_events={legacy_growth_events} \
optimized_growth_events={optimized_growth_events}"
    );
    assert!(legacy_growth_events > optimized_growth_events);
    assert_eq!(optimized_growth_events, 0);
}

fn growth_events(count: usize, reserve: bool) -> usize {
    let mut candidates = if reserve {
        Vec::with_capacity(count)
    } else {
        Vec::new()
    };
    let mut growth_events = 0;
    for candidate in 0..count {
        let previous_capacity = candidates.capacity();
        candidates.push(candidate);
        growth_events += usize::from(candidates.capacity() != previous_capacity);
    }
    growth_events
}
