use crate::ui::curve::{CurveInterpolation, CurveKey, CurvePoint, CurveView};
use zircon_runtime::core::framework::animation::{
    AnimationChannelValueAsset, AnimationInterpolationAsset, AnimationSequenceTrackAsset,
    AnimationTrackPath,
};

use super::{AnimationEditorSession, AnimationEditorSessionError, AnimationSequenceSessionState};

/// The curve-editor projection for the timeline-selected sequence track.
///
/// Vector tracks expand into independently selectable scalar component curves. Discrete and
/// quaternion channels stay in their timeline lane because the runtime evaluates those with
/// non-scalar semantics.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AnimationCurveFoundationView {
    pub selected_track_id: Option<String>,
    pub curves: Vec<CurveView<String>>,
}

impl AnimationEditorSession {
    pub fn curve_foundation(
        &self,
    ) -> Result<AnimationCurveFoundationView, AnimationEditorSessionError> {
        let document = self.document().read();
        let Some(asset) = document.asset().as_sequence() else {
            return Err(AnimationEditorSessionError::new(
                "active animation editor is not a sequence document",
            ));
        };
        let sequence = self.sequence.as_ref().ok_or_else(|| {
            AnimationEditorSessionError::new("sequence source is missing its transient UI state")
        })?;
        project_selected_track_curves(asset, sequence)
    }
}

fn project_selected_track_curves(
    asset: &zircon_runtime::core::framework::animation::AnimationSequenceAsset,
    sequence: &AnimationSequenceSessionState,
) -> Result<AnimationCurveFoundationView, AnimationEditorSessionError> {
    let Some((track_path, _, _)) = sequence.selected_span.as_ref() else {
        return Ok(AnimationCurveFoundationView::default());
    };
    let (entity_path, property_path) = track_path
        .split()
        .map_err(|error| AnimationEditorSessionError::new(error.to_string()))?;
    let track = asset
        .bindings
        .iter()
        .find(|binding| binding.entity_path == entity_path)
        .and_then(|binding| {
            binding
                .tracks
                .iter()
                .find(|track| track.property_path == property_path)
        })
        .ok_or_else(|| {
            AnimationEditorSessionError::new(format!("missing animation track {track_path}"))
        })?;
    let selected_track_id = track_path.to_string();

    Ok(AnimationCurveFoundationView {
        curves: project_track_curves(&selected_track_id, track),
        selected_track_id: Some(selected_track_id),
    })
}

fn project_track_curves(
    track_id: &str,
    track: &AnimationSequenceTrackAsset,
) -> Vec<CurveView<String>> {
    let Some(first_key) = track.channel.keys.first() else {
        return Vec::new();
    };
    let Some(layout) = CurveComponentLayout::from_value(&first_key.value) else {
        return Vec::new();
    };

    layout
        .component_names()
        .iter()
        .enumerate()
        .filter_map(|(component_index, component_name)| {
            project_component_curve(track_id, track, layout, component_index, component_name)
        })
        .collect()
}

fn project_component_curve(
    track_id: &str,
    track: &AnimationSequenceTrackAsset,
    layout: CurveComponentLayout,
    component_index: usize,
    component_name: &str,
) -> Option<CurveView<String>> {
    let keys = track
        .channel
        .keys
        .iter()
        .map(|key| {
            let value = layout.component_value(&key.value, component_index)?;
            if !key.time_seconds.is_finite() || !value.is_finite() {
                return None;
            }
            Some(
                CurveKey::new(
                    format!("{track_id}@{:08x}", key.time_seconds.to_bits()),
                    CurvePoint::new(key.time_seconds, value),
                )
                .with_tangents(
                    key.in_tangent
                        .as_ref()
                        .and_then(|tangent| layout.component_value(tangent, component_index)),
                    key.out_tangent
                        .as_ref()
                        .and_then(|tangent| layout.component_value(tangent, component_index)),
                ),
            )
        })
        .collect::<Option<Vec<_>>>()?;
    Some(CurveView {
        id: format!("{track_id}.{component_name}"),
        display_name: format!("{track_id} {}", component_name.to_ascii_uppercase()),
        interpolation: curve_interpolation(track.channel.interpolation),
        keys,
    })
}

#[derive(Clone, Copy)]
enum CurveComponentLayout {
    Scalar,
    Vec2,
    Vec3,
    Vec4,
}

impl CurveComponentLayout {
    fn from_value(value: &AnimationChannelValueAsset) -> Option<Self> {
        match value {
            AnimationChannelValueAsset::Scalar(_) => Some(Self::Scalar),
            AnimationChannelValueAsset::Vec2(_) => Some(Self::Vec2),
            AnimationChannelValueAsset::Vec3(_) => Some(Self::Vec3),
            AnimationChannelValueAsset::Vec4(_) => Some(Self::Vec4),
            AnimationChannelValueAsset::Bool(_)
            | AnimationChannelValueAsset::Integer(_)
            | AnimationChannelValueAsset::Quaternion(_) => None,
        }
    }

    fn component_names(self) -> &'static [&'static str] {
        match self {
            Self::Scalar => &["value"],
            Self::Vec2 => &["x", "y"],
            Self::Vec3 => &["x", "y", "z"],
            Self::Vec4 => &["x", "y", "z", "w"],
        }
    }

    fn component_value(self, value: &AnimationChannelValueAsset, index: usize) -> Option<f32> {
        match (self, value) {
            (Self::Scalar, AnimationChannelValueAsset::Scalar(value)) if index == 0 => Some(*value),
            (Self::Vec2, AnimationChannelValueAsset::Vec2(value)) => value.get(index).copied(),
            (Self::Vec3, AnimationChannelValueAsset::Vec3(value)) => value.get(index).copied(),
            (Self::Vec4, AnimationChannelValueAsset::Vec4(value)) => value.get(index).copied(),
            _ => None,
        }
    }
}

fn curve_interpolation(interpolation: AnimationInterpolationAsset) -> CurveInterpolation {
    match interpolation {
        AnimationInterpolationAsset::Step => CurveInterpolation::Step,
        AnimationInterpolationAsset::Linear => CurveInterpolation::Linear,
        AnimationInterpolationAsset::Hermite => CurveInterpolation::Hermite,
    }
}
