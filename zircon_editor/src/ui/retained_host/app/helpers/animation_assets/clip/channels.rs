use zircon_runtime::core::framework::animation::{
    AnimationChannelAsset, AnimationChannelKeyAsset, AnimationChannelValueAsset,
    AnimationInterpolationAsset,
};

pub(super) fn map_animation_interpolation(
    interpolation: gltf::animation::Interpolation,
) -> AnimationInterpolationAsset {
    match interpolation {
        gltf::animation::Interpolation::Step => AnimationInterpolationAsset::Step,
        gltf::animation::Interpolation::Linear | gltf::animation::Interpolation::CubicSpline => {
            AnimationInterpolationAsset::Hermite
        }
    }
}

pub(super) fn constant_vec3_channel(value: [f32; 3]) -> AnimationChannelAsset {
    AnimationChannelAsset {
        interpolation: AnimationInterpolationAsset::Step,
        keys: vec![AnimationChannelKeyAsset {
            time_seconds: 0.0,
            value: AnimationChannelValueAsset::Vec3(value),
            in_tangent: None,
            out_tangent: None,
        }],
    }
}

pub(super) fn constant_quaternion_channel(value: [f32; 4]) -> AnimationChannelAsset {
    AnimationChannelAsset {
        interpolation: AnimationInterpolationAsset::Step,
        keys: vec![AnimationChannelKeyAsset {
            time_seconds: 0.0,
            value: AnimationChannelValueAsset::Quaternion(value),
            in_tangent: None,
            out_tangent: None,
        }],
    }
}

pub(super) fn vec3_channel_from_samples(
    times: &[f32],
    values: &[[f32; 3]],
    interpolation: AnimationInterpolationAsset,
) -> Result<AnimationChannelAsset, String> {
    if times.len() != values.len() {
        return Err("gltf animation translation/scaling key count mismatch".to_string());
    }
    Ok(AnimationChannelAsset {
        interpolation,
        keys: times
            .iter()
            .zip(values.iter())
            .map(|(time_seconds, value)| AnimationChannelKeyAsset {
                time_seconds: *time_seconds,
                value: AnimationChannelValueAsset::Vec3(*value),
                in_tangent: matches!(interpolation, AnimationInterpolationAsset::Hermite)
                    .then_some(AnimationChannelValueAsset::Vec3([0.0, 0.0, 0.0])),
                out_tangent: matches!(interpolation, AnimationInterpolationAsset::Hermite)
                    .then_some(AnimationChannelValueAsset::Vec3([0.0, 0.0, 0.0])),
            })
            .collect(),
    })
}

pub(super) fn quaternion_channel_from_samples(
    times: &[f32],
    values: &[[f32; 4]],
    interpolation: AnimationInterpolationAsset,
) -> Result<AnimationChannelAsset, String> {
    if times.len() != values.len() {
        return Err("gltf animation rotation key count mismatch".to_string());
    }
    Ok(AnimationChannelAsset {
        interpolation,
        keys: times
            .iter()
            .zip(values.iter())
            .map(|(time_seconds, value)| AnimationChannelKeyAsset {
                time_seconds: *time_seconds,
                value: AnimationChannelValueAsset::Quaternion(*value),
                in_tangent: None,
                out_tangent: None,
            })
            .collect(),
    })
}
