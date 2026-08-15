use std::path::Path;

use zircon_runtime::core::framework::animation::AnimationTrackPath;
use zircon_runtime::core::framework::animation::{
    AnimationGraphAsset, AnimationSequenceTrackAsset, AnimationStateMachineAsset,
};

use super::{
    AnimationEditorDocument, AnimationEditorSession, AnimationEditorSessionError,
    AnimationSequenceDocument, DEFAULT_SEQUENCE_FRAMES_PER_SECOND,
};

impl AnimationEditorSession {
    pub(super) fn sequence_document_mut(
        &mut self,
    ) -> Result<&mut AnimationSequenceDocument, String> {
        match &mut self.document {
            AnimationEditorDocument::Sequence(document) => Ok(document),
            _ => Err("active animation editor is not a sequence document".to_string()),
        }
    }

    pub(super) fn graph_asset_mut(&mut self) -> Result<&mut AnimationGraphAsset, String> {
        match &mut self.document {
            AnimationEditorDocument::Graph(asset) => Ok(asset),
            _ => Err("active animation editor is not a graph document".to_string()),
        }
    }

    pub(super) fn state_machine_asset_mut(
        &mut self,
    ) -> Result<&mut AnimationStateMachineAsset, String> {
        match &mut self.document {
            AnimationEditorDocument::StateMachine(asset) => Ok(asset),
            _ => Err("active animation editor is not a state-machine document".to_string()),
        }
    }

    pub(super) fn sequence_frames_per_second(&self) -> f32 {
        match &self.document {
            AnimationEditorDocument::Sequence(document) => {
                sanitize_frames_per_second(document.asset.frames_per_second)
            }
            _ => DEFAULT_SEQUENCE_FRAMES_PER_SECOND,
        }
    }

    pub(super) fn sequence_track_mut(
        &mut self,
        track_path: &AnimationTrackPath,
    ) -> Result<Option<&mut AnimationSequenceTrackAsset>, String> {
        let (entity_path, property_path) = track_path.split().map_err(|error| error.to_string())?;
        let document = self.sequence_document_mut()?;
        for binding in &mut document.asset.bindings {
            if binding.entity_path != entity_path {
                continue;
            }
            if let Some(track) = binding
                .tracks
                .iter_mut()
                .find(|track| track.property_path == property_path)
            {
                return Ok(Some(track));
            }
        }
        Ok(None)
    }

    pub(crate) fn document_bytes(&self) -> Result<Vec<u8>, AnimationEditorSessionError> {
        match &self.document {
            AnimationEditorDocument::Sequence(document) => document
                .asset
                .to_bytes()
                .map_err(|error| AnimationEditorSessionError(error.to_string())),
            AnimationEditorDocument::Graph(asset) => asset
                .to_bytes()
                .map_err(|error| AnimationEditorSessionError(error.to_string())),
            AnimationEditorDocument::StateMachine(asset) => asset
                .to_bytes()
                .map_err(|error| AnimationEditorSessionError(error.to_string())),
        }
    }
}

pub(super) fn fallback_title(asset_path: &str) -> String {
    Path::new(asset_path)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .map(str::to_string)
        .unwrap_or_else(|| asset_path.to_string())
}

pub(super) fn duration_frames(duration_seconds: f32, frames_per_second: f32) -> u32 {
    (sanitize_duration_seconds(duration_seconds) * sanitize_frames_per_second(frames_per_second))
        .round() as u32
}

pub(super) fn clamp_timeline_span(
    start_frame: u32,
    end_frame: u32,
    range_start: u32,
    range_end: u32,
) -> (u32, u32) {
    (
        start_frame.clamp(range_start, range_end),
        end_frame.clamp(range_start, range_end),
    )
}

pub(super) fn frame_to_seconds(frame: u32, frames_per_second: f32) -> f32 {
    frame as f32 / sanitize_frames_per_second(frames_per_second).max(1.0)
}

fn sanitize_duration_seconds(duration_seconds: f32) -> f32 {
    if duration_seconds.is_finite() && duration_seconds >= 0.0 {
        duration_seconds
    } else {
        0.0
    }
}

pub(super) fn sanitize_frames_per_second(frames_per_second: f32) -> f32 {
    if frames_per_second.is_finite() && frames_per_second > 0.0 {
        frames_per_second
    } else {
        DEFAULT_SEQUENCE_FRAMES_PER_SECOND
    }
}
