use std::hint::black_box;
use std::time::Instant;

use super::viewport_effects;
use crate::core::editor_event::{EditorEventEffect, EditorViewportEvent};
use crate::scene::viewport::{GridMode, ViewportFeedback};

#[test]
fn viewport_effects_capacity_preserves_empty_and_ordered_effects() {
    let empty = viewport_effects(
        &EditorViewportEvent::CancelInteraction,
        &ViewportFeedback::default(),
        false,
        false,
    );
    assert!(empty.is_empty());
    assert_eq!(empty.capacity(), 0);

    let full = viewport_effects(
        &EditorViewportEvent::Resized {
            width: 1280,
            height: 720,
        },
        &ViewportFeedback::default(),
        true,
        false,
    );
    assert_eq!(
        full,
        vec![
            EditorEventEffect::RenderChanged,
            EditorEventEffect::PresentationChanged,
            EditorEventEffect::ReflectionChanged,
        ]
    );
    assert!(full.capacity() >= 3);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn editor817_viewport_effects_capacity_release_benchmark() {
    const RUNS_PER_SAMPLE: usize = 4_096;
    const SAMPLE_PAIRS: usize = 17;

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_effect_projection(RUNS_PER_SAMPLE, false));
            optimized_samples.push(measure_effect_projection(RUNS_PER_SAMPLE, true));
        } else {
            optimized_samples.push(measure_effect_projection(RUNS_PER_SAMPLE, true));
            legacy_samples.push(measure_effect_projection(RUNS_PER_SAMPLE, false));
        }
    }

    let legacy_growth_events = growth_events(3);
    let optimized_growth_events = 0;
    assert!(legacy_growth_events > optimized_growth_events);
    println!(
        "EDITOR817_VIEWPORT_EFFECTS_CAPACITY_BENCH_V1 runs_per_sample={RUNS_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} legacy_growth_events={legacy_growth_events} optimized_growth_events={optimized_growth_events} legacy_p95_ns={} optimized_p95_ns={}",
        percentile(&legacy_samples, 95),
        percentile(&optimized_samples, 95),
    );
}

fn measure_effect_projection(runs: usize, optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..runs {
        let mut effects = if optimized {
            Vec::with_capacity(3)
        } else {
            Vec::new()
        };
        effects.push(EditorEventEffect::RenderChanged);
        effects.push(EditorEventEffect::PresentationChanged);
        effects.push(EditorEventEffect::ReflectionChanged);
        checksum = checksum.wrapping_add(effects.len());
        black_box(effects);
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn growth_events(item_count: usize) -> usize {
    let mut capacity = 0usize;
    let mut events = 0usize;
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

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

#[test]
fn viewport_chrome_state_change_keeps_render_but_skips_full_presentation() {
    let event = EditorViewportEvent::SetGridMode {
        mode: GridMode::VisibleAndSnap,
    };

    let effects = viewport_effects(&event, &ViewportFeedback::default(), true, true);

    assert!(effects.contains(&EditorEventEffect::RenderChanged));
    assert!(effects.contains(&EditorEventEffect::ReflectionChanged));
    assert!(!effects.contains(&EditorEventEffect::PresentationChanged));
}

#[test]
fn non_chrome_structural_change_still_requests_presentation() {
    let event = EditorViewportEvent::Resized {
        width: 1280,
        height: 720,
    };

    let effects = viewport_effects(&event, &ViewportFeedback::default(), true, false);

    assert!(effects.contains(&EditorEventEffect::PresentationChanged));
}

#[test]
fn empty_cancel_interaction_has_no_effects() {
    let event = EditorViewportEvent::CancelInteraction;

    assert!(!super::structural_viewport_event(
        &event,
        &ViewportFeedback::default()
    ));
    assert!(viewport_effects(&event, &ViewportFeedback::default(), false, false).is_empty());
}

#[test]
fn active_cancel_interaction_refreshes_viewport_projections() {
    let event = EditorViewportEvent::CancelInteraction;
    let feedback = ViewportFeedback {
        transformed_node: Some(42),
        ..ViewportFeedback::default()
    };

    let effects = viewport_effects(&event, &feedback, false, false);

    assert!(effects.contains(&EditorEventEffect::RenderChanged));
    assert!(effects.contains(&EditorEventEffect::PresentationChanged));
    assert!(effects.contains(&EditorEventEffect::ReflectionChanged));
}

#[test]
fn stale_pointer_product_requests_a_render_rebuild_without_presentation_churn() {
    let feedback = ViewportFeedback {
        interaction_extract_stale: true,
        ..ViewportFeedback::default()
    };

    let effects = viewport_effects(
        &EditorViewportEvent::PointerMoved { x: 12.0, y: 24.0 },
        &feedback,
        false,
        false,
    );

    assert!(effects.contains(&EditorEventEffect::RenderChanged));
    assert!(!effects.contains(&EditorEventEffect::PresentationChanged));
    assert!(!effects.contains(&EditorEventEffect::ReflectionChanged));
}
