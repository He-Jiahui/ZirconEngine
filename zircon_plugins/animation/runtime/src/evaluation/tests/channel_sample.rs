use zircon_runtime::core::framework::animation::{
    AnimationChannelAsset, AnimationChannelKeyAsset, AnimationChannelValueAsset,
    AnimationInterpolationAsset,
};

use super::{interior_key_pair, sample_channel};

fn scalar_channel(interpolation: AnimationInterpolationAsset) -> AnimationChannelAsset {
    AnimationChannelAsset {
        interpolation,
        keys: vec![
            AnimationChannelKeyAsset {
                time_seconds: 0.0,
                value: AnimationChannelValueAsset::Scalar(2.0),
                in_tangent: None,
                out_tangent: None,
            },
            AnimationChannelKeyAsset {
                time_seconds: 1.0,
                value: AnimationChannelValueAsset::Scalar(6.0),
                in_tangent: None,
                out_tangent: None,
            },
            AnimationChannelKeyAsset {
                time_seconds: 3.0,
                value: AnimationChannelValueAsset::Scalar(14.0),
                in_tangent: None,
                out_tangent: None,
            },
        ],
    }
}

#[test]
fn interior_lookup_keeps_step_key_boundaries_left_inclusive() {
    let channel = scalar_channel(AnimationInterpolationAsset::Step);
    let (left, right) = interior_key_pair(&channel.keys, 1.0).expect("interior key pair");

    assert_eq!(left.time_seconds, 0.0);
    assert_eq!(right.time_seconds, 1.0);
    assert_eq!(
        sample_channel(&channel, 1.0),
        Some(AnimationChannelValueAsset::Scalar(2.0))
    );
}

#[test]
fn interior_lookup_selects_the_neighbors_for_linear_interpolation() {
    let channel = scalar_channel(AnimationInterpolationAsset::Linear);
    let (left, right) = interior_key_pair(&channel.keys, 2.0).expect("interior key pair");

    assert_eq!(left.time_seconds, 1.0);
    assert_eq!(right.time_seconds, 3.0);
    assert_eq!(
        sample_channel(&channel, 2.0),
        Some(AnimationChannelValueAsset::Scalar(10.0))
    );
}
