mod capability;
mod extension_ids;
mod plugin;

use std::collections::BTreeMap;

use zircon_runtime::core::framework::animation::AnimationSequenceAsset;

pub use capability::{
    ANIMATION_TIMELINE_EVENT_TRACK_CAPABILITY, CAPABILITY, EDITOR_CAPABILITIES, PLUGIN_ID,
};
pub use extension_ids::{
    TIMELINE_SEQUENCE_DRAWER_ID, TIMELINE_SEQUENCE_TEMPLATE_ID, TIMELINE_SEQUENCE_VIEW_ID,
};
pub use plugin::{
    editor_capabilities, editor_plugin, editor_plugin_descriptor, package_manifest,
    plugin_registration, timeline_sequence_dist_module_manifest, TimelineSequenceEditorPlugin,
    TIMELINE_SEQUENCE_DIST_CRATE_NAME, TIMELINE_SEQUENCE_DIST_EDITOR_ENTRY,
};

#[derive(Clone, Debug, PartialEq)]
pub struct TimelineEventMarker {
    pub time_seconds: f32,
    pub event: String,
    pub payload: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TimelineKeyframeMoveRequest {
    pub binding_index: usize,
    pub track_index: usize,
    pub key_index: usize,
    pub new_time_seconds: f32,
}

pub fn validate_timeline_sequence(sequence: &AnimationSequenceAsset) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if sequence.duration_seconds <= 0.0 {
        diagnostics.push("timeline duration must be greater than zero".to_string());
    }
    if sequence.frames_per_second <= 0.0 {
        diagnostics.push("timeline frame rate must be greater than zero".to_string());
    }

    for binding in &sequence.bindings {
        for track in &binding.tracks {
            let mut previous_time = None;
            for key in &track.channel.keys {
                if key.time_seconds < 0.0 || key.time_seconds > sequence.duration_seconds {
                    diagnostics.push(format!(
                        "keyframe `{}` on `{}` is outside timeline range 0..{}",
                        key.time_seconds, track.property_path, sequence.duration_seconds
                    ));
                }
                if let Some(previous_time) = previous_time {
                    if key.time_seconds < previous_time {
                        diagnostics.push(format!(
                            "keyframes on `{}` must be sorted by time",
                            track.property_path
                        ));
                    }
                }
                previous_time = Some(key.time_seconds);
            }
        }
    }

    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

pub fn move_timeline_keyframe(
    sequence: &mut AnimationSequenceAsset,
    request: &TimelineKeyframeMoveRequest,
) -> Result<(), Vec<String>> {
    let mut diagnostics = Vec::new();
    if request.new_time_seconds < 0.0 || request.new_time_seconds > sequence.duration_seconds {
        diagnostics.push(format!(
            "timeline keyframe move target `{}` is outside timeline range 0..{}",
            request.new_time_seconds, sequence.duration_seconds
        ));
    }
    let Some(binding) = sequence.bindings.get_mut(request.binding_index) else {
        diagnostics.push(format!(
            "timeline binding index {} is outside {} bindings",
            request.binding_index,
            sequence.bindings.len()
        ));
        return Err(diagnostics);
    };
    let Some(track) = binding.tracks.get_mut(request.track_index) else {
        diagnostics.push(format!(
            "timeline track index {} is outside {} tracks",
            request.track_index,
            binding.tracks.len()
        ));
        return Err(diagnostics);
    };
    let Some(key) = track.channel.keys.get_mut(request.key_index) else {
        diagnostics.push(format!(
            "timeline keyframe index {} is outside {} keys",
            request.key_index,
            track.channel.keys.len()
        ));
        return Err(diagnostics);
    };
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    key.time_seconds = request.new_time_seconds;
    track
        .channel
        .keys
        .sort_by(|left, right| left.time_seconds.total_cmp(&right.time_seconds));
    let diagnostics = validate_timeline_sequence(sequence);
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

pub fn sorted_timeline_track_paths(sequence: &AnimationSequenceAsset) -> Vec<String> {
    let mut paths = sequence
        .bindings
        .iter()
        .flat_map(|binding| {
            binding
                .tracks
                .iter()
                .map(|track| format!("{}:{}", binding.entity_path, track.property_path))
        })
        .collect::<Vec<_>>();
    paths.sort();
    paths
}

pub fn validate_event_marker_payload(
    marker: &TimelineEventMarker,
    duration_seconds: f32,
) -> Result<(), String> {
    if marker.event.trim().is_empty() {
        return Err("timeline event marker must name an event".to_string());
    }
    if marker.time_seconds < 0.0 || marker.time_seconds > duration_seconds {
        return Err(format!(
            "timeline event marker `{}` is outside timeline range 0..{}",
            marker.event, duration_seconds
        ));
    }
    if marker.payload.keys().any(|key| key.trim().is_empty()) {
        return Err("timeline event marker payload keys must not be empty".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests;
