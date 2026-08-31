use std::path::Path;

use super::{
    AnimationEditorSession, AnimationEditorSessionError, AnimationSequenceSessionState,
    DEFAULT_SEQUENCE_FRAMES_PER_SECOND,
};

impl AnimationEditorSession {
    pub(super) fn sequence_state_mut(
        &mut self,
    ) -> Result<&mut AnimationSequenceSessionState, String> {
        match &mut self.sequence {
            Some(sequence) => Ok(sequence),
            _ => Err("active animation editor is not a sequence document".to_string()),
        }
    }

    pub(crate) fn document_bytes(&self) -> Result<Vec<u8>, AnimationEditorSessionError> {
        self.document()
            .read()
            .document_bytes()
            .map_err(|error| AnimationEditorSessionError::new(error.to_string()))
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
