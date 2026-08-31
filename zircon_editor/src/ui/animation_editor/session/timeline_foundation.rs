use crate::ui::preview_scene::PreviewPlayback;
use crate::ui::timeline::{TimelineKey, TimelineRange, TimelineTrackView};
use zircon_runtime::core::framework::animation::{
    AnimationChannelValueAsset, AnimationSequenceTrackAsset,
};

use super::support::{frame_to_seconds, sanitize_frames_per_second};
use super::{AnimationEditorSession, AnimationEditorSessionError, AnimationSequenceSessionState};

/// A sequence-session projection consumed by the shared time and preview foundations.
///
/// It borrows no mutable UI state and contains no copied `AnimationSequenceAsset`; each call
/// projects the current runtime-owned asset plus editor-local range/playback state.
#[derive(Clone, Debug, PartialEq)]
pub struct AnimationTimelineFoundationView {
    pub range: TimelineRange,
    pub playhead: f32,
    pub playback: PreviewPlayback,
    pub tracks: Vec<TimelineTrackView<String>>,
}

impl AnimationEditorSession {
    pub fn timeline_foundation(
        &self,
    ) -> Result<AnimationTimelineFoundationView, AnimationEditorSessionError> {
        let document = self.document().read();
        let Some(asset) = document.asset().as_sequence() else {
            return Err(AnimationEditorSessionError::new(
                "active animation editor is not a sequence document",
            ));
        };
        let sequence = self.sequence.as_ref().ok_or_else(|| {
            AnimationEditorSessionError::new("sequence source is missing its transient UI state")
        })?;
        Ok(project_sequence_timeline(asset, sequence))
    }
}

pub(super) fn project_sequence_timeline(
    asset: &zircon_runtime::core::framework::animation::AnimationSequenceAsset,
    sequence: &AnimationSequenceSessionState,
) -> AnimationTimelineFoundationView {
    let frames_per_second = sanitize_frames_per_second(asset.frames_per_second);
    AnimationTimelineFoundationView {
        range: TimelineRange::new(
            frame_to_seconds(sequence.timeline_start_frame, frames_per_second),
            frame_to_seconds(sequence.timeline_end_frame, frames_per_second),
        ),
        playhead: frame_to_seconds(sequence.current_frame, frames_per_second),
        playback: PreviewPlayback::new(
            sequence.playing,
            sequence.looping,
            sequence.speed,
            frame_to_seconds(sequence.current_frame, frames_per_second),
        ),
        tracks: asset
            .bindings
            .iter()
            .flat_map(|binding| {
                binding.tracks.iter().map(move |track| {
                    let path = zircon_runtime::core::framework::animation::AnimationTrackPath::new(
                        binding.entity_path.clone(),
                        track.property_path.clone(),
                    );
                    let track_id = path.to_string();
                    TimelineTrackView {
                        id: track_id.clone(),
                        display_name: track_id.clone(),
                        value_kind: track_value_kind(track).to_string(),
                        keys: track
                            .channel
                            .keys
                            .iter()
                            .map(|key| {
                                TimelineKey::new(
                                    format!("{track_id}@{:08x}", key.time_seconds.to_bits()),
                                    key.time_seconds,
                                    format!("{:.3}s", key.time_seconds),
                                )
                            })
                            .collect(),
                        sections: Vec::new(),
                    }
                })
            })
            .collect(),
    }
}

fn track_value_kind(track: &AnimationSequenceTrackAsset) -> &'static str {
    match track.channel.keys.first().map(|key| &key.value) {
        Some(AnimationChannelValueAsset::Bool(_)) => "bool",
        Some(AnimationChannelValueAsset::Integer(_)) => "integer",
        Some(AnimationChannelValueAsset::Scalar(_)) => "float",
        Some(AnimationChannelValueAsset::Vec2(_)) => "vector2",
        Some(AnimationChannelValueAsset::Vec3(_)) => "vector3",
        Some(AnimationChannelValueAsset::Vec4(_)) => "vector4",
        Some(AnimationChannelValueAsset::Quaternion(_)) => "quaternion",
        None => "untyped",
    }
}
