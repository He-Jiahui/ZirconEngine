//! Structural and semantic validation for sequence authoring assets.

use std::collections::BTreeSet;

use crate::core::framework::animation::{
    AnimationChannelAsset, AnimationChannelValueAsset, AnimationInterpolationAsset,
    AnimationSequenceAsset, AnimationSequenceBindingAsset, AnimationSequenceTrackAsset,
};
use crate::core::math::Real;

use super::model::{
    AnimationCompiledSequence, AnimationCompiledSequenceBinding, AnimationCompiledSequenceKey,
    AnimationCompiledSequenceTrack, AnimationCompiledSequenceValueKind,
    AnimationSequenceCompilation,
};
use crate::core::framework::animation::compiler::{
    AnimationCompileDiagnostic, AnimationCompileElement, AnimationCompileSeverity,
};

const INVALID_DURATION: &str = "ZR-ANIM-COMP-SEQUENCE-001";
const INVALID_FRAMES_PER_SECOND: &str = "ZR-ANIM-COMP-SEQUENCE-002";
const EMPTY_TARGET_ID: &str = "ZR-ANIM-COMP-SEQUENCE-003";
const DUPLICATE_TRACK: &str = "ZR-ANIM-COMP-SEQUENCE-004";
const EMPTY_CHANNEL: &str = "ZR-ANIM-COMP-SEQUENCE-005";
const NON_FINITE_KEY_TIME: &str = "ZR-ANIM-COMP-SEQUENCE-006";
const OUT_OF_RANGE_KEY_TIME: &str = "ZR-ANIM-COMP-SEQUENCE-007";
const NON_MONOTONIC_KEY_TIME: &str = "ZR-ANIM-COMP-SEQUENCE-008";
const INCONSISTENT_VALUE_TYPE: &str = "ZR-ANIM-COMP-SEQUENCE-009";
const NON_FINITE_VALUE: &str = "ZR-ANIM-COMP-SEQUENCE-010";
const INVALID_QUATERNION: &str = "ZR-ANIM-COMP-SEQUENCE-011";
const INVALID_INTERPOLATION_DOMAIN: &str = "ZR-ANIM-COMP-SEQUENCE-012";
const INVALID_HERMITE_TANGENT: &str = "ZR-ANIM-COMP-SEQUENCE-013";

/// Validates a sequence asset and lowers its source tracks to canonical increasing-time IR.
///
/// Entity/property binding resolution belongs to the world-specific sequence compiler. This
/// source-only phase protects that later compiler from malformed time/value/channel semantics.
pub fn compile_animation_sequence(asset: &AnimationSequenceAsset) -> AnimationSequenceCompilation {
    let mut diagnostics = Vec::new();
    let duration_valid = asset.duration_seconds.is_finite() && asset.duration_seconds >= 0.0;
    if !duration_valid {
        push_error(
            &mut diagnostics,
            INVALID_DURATION,
            AnimationCompileElement::Asset,
            "sequence duration must be finite and non-negative",
        );
    }
    if !asset.frames_per_second.is_finite() || asset.frames_per_second <= Real::EPSILON {
        push_error(
            &mut diagnostics,
            INVALID_FRAMES_PER_SECOND,
            AnimationCompileElement::Asset,
            "sequence frames per second must be finite and greater than zero",
        );
    }

    let bindings = asset
        .bindings
        .iter()
        .enumerate()
        .map(|(binding_index, binding)| {
            compile_binding(
                binding_index,
                binding,
                asset.duration_seconds,
                duration_valid,
                &mut diagnostics,
            )
        })
        .collect();

    if diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity() == AnimationCompileSeverity::Error)
    {
        return AnimationSequenceCompilation::new(None, diagnostics);
    }

    AnimationSequenceCompilation::new(
        Some(AnimationCompiledSequence::new(
            asset.duration_seconds,
            asset.frames_per_second,
            bindings,
        )),
        diagnostics,
    )
}

fn compile_binding(
    binding_index: usize,
    binding: &AnimationSequenceBindingAsset,
    duration_seconds: Real,
    duration_valid: bool,
    diagnostics: &mut Vec<AnimationCompileDiagnostic>,
) -> AnimationCompiledSequenceBinding {
    let binding_element = AnimationCompileElement::SequenceBinding { binding_index };
    if binding
        .target_id
        .as_deref()
        .is_some_and(|target_id| target_id.trim().is_empty())
    {
        push_error(
            diagnostics,
            EMPTY_TARGET_ID,
            binding_element,
            "sequence target id must not be empty or whitespace when present",
        );
    }

    let mut property_paths = BTreeSet::new();
    let tracks = binding
        .tracks
        .iter()
        .enumerate()
        .map(|(track_index, track)| {
            let property_path = track.property_path.as_str();
            if !property_paths.insert(property_path) {
                push_error(
                    diagnostics,
                    DUPLICATE_TRACK,
                    track_element(binding_index, track_index, property_path),
                    "a sequence binding must not write the same property from multiple tracks",
                );
            }
            compile_track(
                binding_index,
                track_index,
                track,
                duration_seconds,
                duration_valid,
                diagnostics,
            )
        })
        .collect();

    AnimationCompiledSequenceBinding::new(
        binding.entity_path.clone(),
        binding.target_id.clone(),
        tracks,
    )
}

fn compile_track(
    binding_index: usize,
    track_index: usize,
    track: &AnimationSequenceTrackAsset,
    duration_seconds: Real,
    duration_valid: bool,
    diagnostics: &mut Vec<AnimationCompileDiagnostic>,
) -> AnimationCompiledSequenceTrack {
    let element = track_element(binding_index, track_index, track.property_path.as_str());
    let channel = &track.channel;
    if channel.keys.is_empty() {
        push_error(
            diagnostics,
            EMPTY_CHANNEL,
            element.clone(),
            "sequence channels must contain at least one key",
        );
    }
    let track_value_kind = channel
        .keys
        .first()
        .map(|key| value_kind(&key.value))
        .unwrap_or(AnimationCompiledSequenceValueKind::Scalar);

    let mut previous_time = None;
    let keys = channel
        .keys
        .iter()
        .enumerate()
        .map(|(key_index, key)| {
            let key_element = AnimationCompileElement::SequenceKey {
                binding_index,
                track_index,
                key_index,
            };
            validate_key_time(
                key.time_seconds,
                previous_time,
                duration_seconds,
                duration_valid,
                diagnostics,
                key_element.clone(),
            );
            previous_time = Some(key.time_seconds);
            let key_kind = value_kind(&key.value);
            if key_kind != track_value_kind {
                push_error(
                    diagnostics,
                    INCONSISTENT_VALUE_TYPE,
                    key_element.clone(),
                    "all keys in a sequence track must use one value type",
                );
            }
            validate_value(&key.value, diagnostics, key_element.clone());
            if channel.interpolation == AnimationInterpolationAsset::Hermite {
                validate_hermite_tangent(
                    key.in_tangent.as_ref(),
                    track_value_kind,
                    diagnostics,
                    key_element.clone(),
                );
                validate_hermite_tangent(
                    key.out_tangent.as_ref(),
                    track_value_kind,
                    diagnostics,
                    key_element,
                );
            }
            AnimationCompiledSequenceKey::new(
                key.time_seconds,
                key.value.clone(),
                key.in_tangent.clone(),
                key.out_tangent.clone(),
            )
        })
        .collect();

    if channel.keys.len() > 1
        && channel.interpolation != AnimationInterpolationAsset::Step
        && !value_kind_is_interpolable(track_value_kind)
    {
        push_error(
            diagnostics,
            INVALID_INTERPOLATION_DOMAIN,
            element,
            "linear and Hermite channels require scalar, vector, or quaternion values",
        );
    }

    AnimationCompiledSequenceTrack::new(
        track.property_path.clone(),
        channel.interpolation,
        track_value_kind,
        keys,
    )
}

fn validate_key_time(
    time_seconds: Real,
    previous_time: Option<Real>,
    duration_seconds: Real,
    duration_valid: bool,
    diagnostics: &mut Vec<AnimationCompileDiagnostic>,
    element: AnimationCompileElement,
) {
    if !time_seconds.is_finite() {
        push_error(
            diagnostics,
            NON_FINITE_KEY_TIME,
            element,
            "sequence key time must be finite",
        );
        return;
    }
    if duration_valid && (time_seconds < 0.0 || time_seconds > duration_seconds) {
        push_error(
            diagnostics,
            OUT_OF_RANGE_KEY_TIME,
            element.clone(),
            "sequence key time must be inside the inclusive sequence duration range",
        );
    }
    if previous_time.is_some_and(|previous| time_seconds <= previous) {
        push_error(
            diagnostics,
            NON_MONOTONIC_KEY_TIME,
            element,
            "sequence key times must be strictly increasing",
        );
    }
}

fn validate_value(
    value: &AnimationChannelValueAsset,
    diagnostics: &mut Vec<AnimationCompileDiagnostic>,
    element: AnimationCompileElement,
) {
    if !channel_value_is_finite(value) {
        push_error(
            diagnostics,
            NON_FINITE_VALUE,
            element.clone(),
            "sequence channel values must be finite",
        );
    }
    if let AnimationChannelValueAsset::Quaternion(value) = value {
        if !quaternion_is_normalizable(value) {
            push_error(
                diagnostics,
                INVALID_QUATERNION,
                element,
                "sequence quaternion values must have non-zero length",
            );
        }
    }
}

fn validate_hermite_tangent(
    tangent: Option<&AnimationChannelValueAsset>,
    value_kind: AnimationCompiledSequenceValueKind,
    diagnostics: &mut Vec<AnimationCompileDiagnostic>,
    element: AnimationCompileElement,
) {
    let Some(tangent) = tangent else {
        return;
    };
    if !channel_value_is_finite(tangent) || !tangent_is_compatible(value_kind, tangent) {
        push_error(
            diagnostics,
            INVALID_HERMITE_TANGENT,
            element,
            "Hermite tangents must be finite and compatible with the track value type",
        );
    }
}

fn value_kind(value: &AnimationChannelValueAsset) -> AnimationCompiledSequenceValueKind {
    match value {
        AnimationChannelValueAsset::Bool(_) => AnimationCompiledSequenceValueKind::Bool,
        AnimationChannelValueAsset::Integer(_) => AnimationCompiledSequenceValueKind::Integer,
        AnimationChannelValueAsset::Scalar(_) => AnimationCompiledSequenceValueKind::Scalar,
        AnimationChannelValueAsset::Vec2(_) => AnimationCompiledSequenceValueKind::Vec2,
        AnimationChannelValueAsset::Vec3(_) => AnimationCompiledSequenceValueKind::Vec3,
        AnimationChannelValueAsset::Vec4(_) => AnimationCompiledSequenceValueKind::Vec4,
        AnimationChannelValueAsset::Quaternion(_) => AnimationCompiledSequenceValueKind::Quaternion,
    }
}

fn channel_value_is_finite(value: &AnimationChannelValueAsset) -> bool {
    match value {
        AnimationChannelValueAsset::Bool(_) | AnimationChannelValueAsset::Integer(_) => true,
        AnimationChannelValueAsset::Scalar(value) => value.is_finite(),
        AnimationChannelValueAsset::Vec2(value) => {
            value.iter().all(|component| component.is_finite())
        }
        AnimationChannelValueAsset::Vec3(value) => {
            value.iter().all(|component| component.is_finite())
        }
        AnimationChannelValueAsset::Vec4(value) | AnimationChannelValueAsset::Quaternion(value) => {
            value.iter().all(|component| component.is_finite())
        }
    }
}

fn quaternion_is_normalizable(value: &[Real; 4]) -> bool {
    value
        .iter()
        .map(|component| component * component)
        .sum::<Real>()
        > Real::EPSILON
}

fn value_kind_is_interpolable(kind: AnimationCompiledSequenceValueKind) -> bool {
    matches!(
        kind,
        AnimationCompiledSequenceValueKind::Scalar
            | AnimationCompiledSequenceValueKind::Vec2
            | AnimationCompiledSequenceValueKind::Vec3
            | AnimationCompiledSequenceValueKind::Vec4
            | AnimationCompiledSequenceValueKind::Quaternion
    )
}

fn tangent_is_compatible(
    kind: AnimationCompiledSequenceValueKind,
    tangent: &AnimationChannelValueAsset,
) -> bool {
    matches!(
        (kind, tangent),
        (
            AnimationCompiledSequenceValueKind::Scalar,
            AnimationChannelValueAsset::Scalar(_)
        ) | (
            AnimationCompiledSequenceValueKind::Vec2,
            AnimationChannelValueAsset::Vec2(_)
        ) | (
            AnimationCompiledSequenceValueKind::Vec3,
            AnimationChannelValueAsset::Vec3(_)
        ) | (
            AnimationCompiledSequenceValueKind::Vec4
                | AnimationCompiledSequenceValueKind::Quaternion,
            AnimationChannelValueAsset::Vec4(_) | AnimationChannelValueAsset::Quaternion(_)
        )
    )
}

fn track_element(
    binding_index: usize,
    track_index: usize,
    property_path: &str,
) -> AnimationCompileElement {
    AnimationCompileElement::SequenceTrack {
        binding_index,
        track_index,
        property_path: property_path.to_string(),
    }
}

fn push_error(
    diagnostics: &mut Vec<AnimationCompileDiagnostic>,
    code: &'static str,
    element: AnimationCompileElement,
    message: impl Into<String>,
) {
    diagnostics.push(AnimationCompileDiagnostic::new(
        code,
        AnimationCompileSeverity::Error,
        element,
        message,
    ));
}
