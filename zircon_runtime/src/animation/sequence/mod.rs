use crate::asset::{
    AnimationChannelAsset, AnimationChannelKeyAsset, AnimationChannelValueAsset,
    AnimationInterpolationAsset, AnimationSequenceAsset,
};
use crate::core::framework::animation::AnimationTrackPath;
use crate::core::framework::scene::ScenePropertyValue;
use crate::core::math::{Quat, Real};
use crate::scene::world::World;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AnimationSequenceApplyReport {
    pub applied_tracks: Vec<AnimationTrackPath>,
    pub missing_tracks: Vec<AnimationTrackPath>,
}

pub fn apply_sequence_to_world(
    world: &mut World,
    sequence: &AnimationSequenceAsset,
    time_seconds: Real,
    looping: bool,
) -> Result<AnimationSequenceApplyReport, String> {
    let mut report = AnimationSequenceApplyReport::default();
    let sample_time = resolve_sequence_sample_time(sequence.duration_seconds, time_seconds, looping);

    for binding in &sequence.bindings {
        let Some(entity) = world.resolve_entity_path(&binding.entity_path) else {
            report
                .missing_tracks
                .extend(binding.tracks.iter().map(|track| {
                    AnimationTrackPath::new(
                        binding.entity_path.clone(),
                        track.property_path.clone(),
                    )
                }));
            continue;
        };

        for track in &binding.tracks {
            let track_path =
                AnimationTrackPath::new(binding.entity_path.clone(), track.property_path.clone());
            let Some(sample) = track.channel.sample(sample_time) else {
                report.missing_tracks.push(track_path);
                continue;
            };
            let value = scene_property_value_from_channel(&sample)?;
            match world.set_property(entity, &track.property_path, value) {
                Ok(_) => report.applied_tracks.push(track_path),
                Err(_) => report.missing_tracks.push(track_path),
            }
        }
    }

    Ok(report)
}

fn resolve_sequence_sample_time(duration_seconds: Real, time_seconds: Real, looping: bool) -> Real {
    if !duration_seconds.is_finite() || duration_seconds <= Real::EPSILON {
        return 0.0;
    }

    if !time_seconds.is_finite() {
        return 0.0;
    }

    let clamped = time_seconds.max(0.0);
    if looping {
        if clamped <= duration_seconds {
            clamped
        } else {
            clamped.rem_euclid(duration_seconds)
        }
    } else {
        clamped.min(duration_seconds)
    }
}

pub(crate) trait AnimationChannelSampleExt {
    fn sample(&self, time_seconds: Real) -> Option<AnimationChannelValueAsset>;
}

impl AnimationChannelSampleExt for AnimationChannelAsset {
    fn sample(&self, time_seconds: Real) -> Option<AnimationChannelValueAsset> {
        if !time_seconds.is_finite() || self.keys.iter().any(|key| !key.time_seconds.is_finite()) {
            return None;
        }

        let first = self.keys.first()?;
        if self.keys.len() == 1 || time_seconds <= first.time_seconds {
            return Some(first.value.clone());
        }
        let last = self.keys.last()?;
        if time_seconds >= last.time_seconds {
            return Some(last.value.clone());
        }

        for pair in self.keys.windows(2) {
            let left = &pair[0];
            let right = &pair[1];
            if time_seconds < left.time_seconds || time_seconds > right.time_seconds {
                continue;
            }
            return Some(match self.interpolation {
                AnimationInterpolationAsset::Step => left.value.clone(),
                AnimationInterpolationAsset::Hermite => sample_hermite(left, right, time_seconds),
            });
        }

        Some(last.value.clone())
    }
}

fn scene_property_value_from_channel(
    value: &AnimationChannelValueAsset,
) -> Result<ScenePropertyValue, String> {
    if !animation_channel_value_is_finite(value) {
        return Err(format!("non-finite animation channel sample: {value:?}"));
    }
    if let AnimationChannelValueAsset::Quaternion(value) = value {
        if !quaternion_array_is_normalizable(value) {
            return Err(format!(
                "zero-length quaternion animation channel sample: {value:?}"
            ));
        }
    }

    Ok(match value {
        AnimationChannelValueAsset::Bool(value) => ScenePropertyValue::Bool(*value),
        AnimationChannelValueAsset::Integer(value) => ScenePropertyValue::Integer(*value as i64),
        AnimationChannelValueAsset::Scalar(value) => ScenePropertyValue::Scalar(*value),
        AnimationChannelValueAsset::Vec2(value) => ScenePropertyValue::Vec2(*value),
        AnimationChannelValueAsset::Vec3(value) => ScenePropertyValue::Vec3(*value),
        AnimationChannelValueAsset::Vec4(value) => ScenePropertyValue::Vec4(*value),
        AnimationChannelValueAsset::Quaternion(value) => ScenePropertyValue::Quaternion(*value),
    })
}

fn animation_channel_value_is_finite(value: &AnimationChannelValueAsset) -> bool {
    match value {
        AnimationChannelValueAsset::Bool(_) | AnimationChannelValueAsset::Integer(_) => true,
        AnimationChannelValueAsset::Scalar(value) => value.is_finite(),
        AnimationChannelValueAsset::Vec2(value) => value.iter().all(|component| component.is_finite()),
        AnimationChannelValueAsset::Vec3(value) => value.iter().all(|component| component.is_finite()),
        AnimationChannelValueAsset::Vec4(value) | AnimationChannelValueAsset::Quaternion(value) => {
            value.iter().all(|component| component.is_finite())
        }
    }
}

fn quaternion_array_is_normalizable(value: &[Real; 4]) -> bool {
    value
        .iter()
        .map(|component| component * component)
        .sum::<Real>()
        > Real::EPSILON
}

fn sample_hermite(
    left: &AnimationChannelKeyAsset,
    right: &AnimationChannelKeyAsset,
    time_seconds: Real,
) -> AnimationChannelValueAsset {
    let duration = (right.time_seconds - left.time_seconds).max(Real::EPSILON);
    let t = ((time_seconds - left.time_seconds) / duration).clamp(0.0, 1.0);

    match (&left.value, &right.value) {
        (
            AnimationChannelValueAsset::Scalar(left_value),
            AnimationChannelValueAsset::Scalar(right_value),
        ) => {
            let left_tangent = tangent_scalar(left.out_tangent.as_ref());
            let right_tangent = tangent_scalar(right.in_tangent.as_ref());
            AnimationChannelValueAsset::Scalar(hermite_scalar(
                *left_value,
                left_tangent,
                *right_value,
                right_tangent,
                duration,
                t,
            ))
        }
        (
            AnimationChannelValueAsset::Vec2(left_value),
            AnimationChannelValueAsset::Vec2(right_value),
        ) => AnimationChannelValueAsset::Vec2(hermite_array(
            left_value,
            tangent_array_2(left.out_tangent.as_ref()),
            right_value,
            tangent_array_2(right.in_tangent.as_ref()),
            duration,
            t,
        )),
        (
            AnimationChannelValueAsset::Vec3(left_value),
            AnimationChannelValueAsset::Vec3(right_value),
        ) => AnimationChannelValueAsset::Vec3(hermite_array(
            left_value,
            tangent_array_3(left.out_tangent.as_ref()),
            right_value,
            tangent_array_3(right.in_tangent.as_ref()),
            duration,
            t,
        )),
        (
            AnimationChannelValueAsset::Vec4(left_value),
            AnimationChannelValueAsset::Vec4(right_value),
        ) => AnimationChannelValueAsset::Vec4(hermite_array(
            left_value,
            tangent_array_4(left.out_tangent.as_ref()),
            right_value,
            tangent_array_4(right.in_tangent.as_ref()),
            duration,
            t,
        )),
        (
            AnimationChannelValueAsset::Quaternion(left_value),
            AnimationChannelValueAsset::Quaternion(right_value),
        ) => {
            let left_quat = Quat::from_array(*left_value).normalize();
            let right_quat = Quat::from_array(*right_value).normalize();
            AnimationChannelValueAsset::Quaternion(
                left_quat.slerp(right_quat, t).normalize().to_array(),
            )
        }
        _ => left.value.clone(),
    }
}

fn hermite_scalar(
    left_value: Real,
    left_tangent: Real,
    right_value: Real,
    right_tangent: Real,
    duration: Real,
    t: Real,
) -> Real {
    let t2 = t * t;
    let t3 = t2 * t;
    let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
    let h10 = t3 - 2.0 * t2 + t;
    let h01 = -2.0 * t3 + 3.0 * t2;
    let h11 = t3 - t2;
    h00 * left_value
        + h10 * left_tangent * duration
        + h01 * right_value
        + h11 * right_tangent * duration
}

fn hermite_array<const N: usize>(
    left_value: &[Real; N],
    left_tangent: [Real; N],
    right_value: &[Real; N],
    right_tangent: [Real; N],
    duration: Real,
    t: Real,
) -> [Real; N] {
    let mut result = [0.0; N];
    let mut index = 0;
    while index < N {
        result[index] = hermite_scalar(
            left_value[index],
            left_tangent[index],
            right_value[index],
            right_tangent[index],
            duration,
            t,
        );
        index += 1;
    }
    result
}

fn tangent_scalar(value: Option<&AnimationChannelValueAsset>) -> Real {
    match value {
        Some(AnimationChannelValueAsset::Scalar(value)) => *value,
        _ => 0.0,
    }
}

fn tangent_array_2(value: Option<&AnimationChannelValueAsset>) -> [Real; 2] {
    match value {
        Some(AnimationChannelValueAsset::Vec2(value)) => *value,
        _ => [0.0; 2],
    }
}

fn tangent_array_3(value: Option<&AnimationChannelValueAsset>) -> [Real; 3] {
    match value {
        Some(AnimationChannelValueAsset::Vec3(value)) => *value,
        _ => [0.0; 3],
    }
}

fn tangent_array_4(value: Option<&AnimationChannelValueAsset>) -> [Real; 4] {
    match value {
        Some(AnimationChannelValueAsset::Vec4(value)) => *value,
        Some(AnimationChannelValueAsset::Quaternion(value)) => *value,
        _ => [0.0; 4],
    }
}
