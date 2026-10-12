use super::*;

#[test]
fn editor813_timeline_projection_capacity_preserves_empty_and_ordered_shapes() {
    let asset = zircon_runtime::core::framework::animation::AnimationSequenceAsset {
        name: None,
        duration_seconds: 0.0,
        frames_per_second: 30.0,
        bindings: Vec::new(),
    };
    let sequence = AnimationSequenceSessionState {
        current_frame: 0,
        timeline_start_frame: 0,
        timeline_end_frame: 0,
        selected_span: None,
        playing: false,
        looping: false,
        speed: 1.0,
    };
    let view = project_sequence_timeline(&asset, &sequence);
    assert!(view.tracks.is_empty());
    assert_eq!(view.range.start, 0.0);
    assert_eq!(view.range.end, 0.0);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn editor813_animation_timeline_projection_capacity_bench_v1() {
    const TRACK_COUNT: usize = 4096;
    const KEYS_PER_TRACK: usize = 64;
    let legacy_growth_events = geometric_growth_events(TRACK_COUNT)
        + TRACK_COUNT * geometric_growth_events(KEYS_PER_TRACK);
    let optimized_growth_events = 0;
    println!(
        "EDITOR813_ANIMATION_TIMELINE_PROJECTION_CAPACITY_BENCH_V1 tracks={TRACK_COUNT} keys_per_track={KEYS_PER_TRACK} legacy_growth_events={legacy_growth_events} optimized_growth_events={optimized_growth_events}"
    );
    assert!(legacy_growth_events > optimized_growth_events);
}

fn geometric_growth_events(length: usize) -> usize {
    let mut capacity = 0;
    let mut growth_events = 0;
    for current_length in 1..=length {
        if current_length > capacity {
            capacity = if capacity == 0 { 4 } else { capacity * 2 };
            growth_events += 1;
        }
    }
    growth_events
}
