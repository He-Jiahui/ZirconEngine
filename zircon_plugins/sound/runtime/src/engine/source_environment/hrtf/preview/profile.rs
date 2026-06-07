use zircon_runtime::core::framework::sound::{SoundListenerDescriptor, SoundSourceDescriptor};

use super::super::super::super::math::{add3, length3, scale3, sub3};
use super::super::super::constants::{
    HRTF_PREVIEW_GAIN_MAX, HRTF_PREVIEW_GAIN_MIN, HRTF_PREVIEW_MAX_DELAY_FRAMES,
    SPEED_OF_SOUND_METERS_PER_SECOND,
};

#[derive(Clone, Copy, Debug)]
pub(super) struct HrtfPreviewProfile {
    pub(super) left_gain: f32,
    pub(super) right_gain: f32,
    pub(super) left_delay_frames: usize,
    pub(super) right_delay_frames: usize,
}

impl HrtfPreviewProfile {
    fn identity() -> Self {
        Self {
            left_gain: 1.0,
            right_gain: 1.0,
            left_delay_frames: 0,
            right_delay_frames: 0,
        }
    }

    pub(super) fn is_identity(&self) -> bool {
        self.left_gain == 1.0
            && self.right_gain == 1.0
            && self.left_delay_frames == 0
            && self.right_delay_frames == 0
    }
}

pub(super) fn hrtf_preview_profile(
    source: &SoundSourceDescriptor,
    listener: &SoundListenerDescriptor,
    sample_rate_hz: u32,
    blend: f32,
    spatial_scale: f32,
) -> HrtfPreviewProfile {
    if listener
        .hrtf_profile
        .as_deref()
        .unwrap_or_default()
        .is_empty()
    {
        return HrtfPreviewProfile::identity();
    }

    let source_position = scale3(source.position, spatial_scale);
    let listener_position = scale3(listener.position, spatial_scale);
    let left_ear = add3(
        listener_position,
        scale3(listener.left_ear_offset, spatial_scale),
    );
    let right_ear = add3(
        listener_position,
        scale3(listener.right_ear_offset, spatial_scale),
    );
    let left_distance = length3(sub3(source_position, left_ear)).max(0.0001);
    let right_distance = length3(sub3(source_position, right_ear)).max(0.0001);
    let average_distance = (left_distance + right_distance) * 0.5;
    let left_gain =
        (average_distance / left_distance).clamp(HRTF_PREVIEW_GAIN_MIN, HRTF_PREVIEW_GAIN_MAX);
    let right_gain =
        (average_distance / right_distance).clamp(HRTF_PREVIEW_GAIN_MIN, HRTF_PREVIEW_GAIN_MAX);
    let delay_frames = (((left_distance - right_distance).abs() / SPEED_OF_SOUND_METERS_PER_SECOND)
        * sample_rate_hz.max(1) as f32)
        .round() as usize;
    let delay_frames = delay_frames.min(HRTF_PREVIEW_MAX_DELAY_FRAMES);
    let (left_delay_frames, right_delay_frames) = if left_distance > right_distance {
        (delay_frames, 0)
    } else {
        (0, delay_frames)
    };

    HrtfPreviewProfile {
        left_gain: 1.0 + (left_gain - 1.0) * blend,
        right_gain: 1.0 + (right_gain - 1.0) * blend,
        left_delay_frames,
        right_delay_frames,
    }
}
