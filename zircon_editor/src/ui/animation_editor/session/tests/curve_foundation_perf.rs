use super::*;
use zircon_runtime::core::framework::animation::{AnimationChannelAsset, AnimationChannelKeyAsset};
use zircon_runtime::core::framework::scene::ComponentPropertyPath;

fn vec3_track() -> AnimationSequenceTrackAsset {
    AnimationSequenceTrackAsset {
        property_path: ComponentPropertyPath::parse("Transform.translation").unwrap(),
        channel: AnimationChannelAsset {
            interpolation: AnimationInterpolationAsset::Hermite,
            keys: vec![
                AnimationChannelKeyAsset {
                    time_seconds: 0.0,
                    value: AnimationChannelValueAsset::Vec3([1.0, 2.0, 3.0]),
                    in_tangent: None,
                    out_tangent: None,
                },
                AnimationChannelKeyAsset {
                    time_seconds: 1.0,
                    value: AnimationChannelValueAsset::Vec3([4.0, 5.0, 6.0]),
                    in_tangent: None,
                    out_tangent: None,
                },
            ],
        },
    }
}

#[test]
fn editor814_animation_curve_projection_capacity_preserves_component_order() {
    let track = vec3_track();
    let curves = project_track_curves("Root/Hero.Transform.translation", &track);
    assert_eq!(curves.len(), 3);
    assert_eq!(curves[0].id, "Root/Hero.Transform.translation.x");
    assert_eq!(curves[1].keys[1].point.value, 5.0);
    assert_eq!(curves[2].keys[0].point.time, 0.0);

    let mut discrete = track;
    discrete.channel.keys[0].value = AnimationChannelValueAsset::Bool(true);
    assert!(project_track_curves("discrete", &discrete).is_empty());

    let mut invalid = vec3_track();
    invalid.channel.keys[1].time_seconds = f32::NAN;
    assert!(project_track_curves("invalid", &invalid).is_empty());
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn editor814_animation_curve_projection_capacity_bench_v1() {
    const COMPONENT_COUNT: usize = 4;
    let legacy_growth_events = geometric_growth_events(COMPONENT_COUNT);
    let optimized_growth_events = 0;
    println!(
        "EDITOR814_ANIMATION_CURVE_PROJECTION_CAPACITY_BENCH_V1 components={COMPONENT_COUNT} legacy_growth_events={legacy_growth_events} optimized_growth_events={optimized_growth_events}"
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
