use zircon_runtime::core::framework::sound::{SoundListenerDescriptor, SoundSourceDescriptor};

use super::super::super::math::{dot3, normalize3};
use super::super::constants::{MAX_DOPPLER_PREVIEW_GAIN_OFFSET, SPEED_OF_SOUND_METERS_PER_SECOND};

pub(super) fn doppler_preview_gain(
    source: &SoundSourceDescriptor,
    listener: &SoundListenerDescriptor,
    listener_to_source: [f32; 3],
) -> f32 {
    if !listener.doppler_tracking || source.spatial.doppler_factor <= 0.0 {
        return 1.0;
    }
    let direction_to_listener = normalize3([
        -listener_to_source[0],
        -listener_to_source[1],
        -listener_to_source[2],
    ]);
    let source_velocity = dot3(source.velocity, direction_to_listener);
    let listener_velocity = dot3(listener.velocity, direction_to_listener);
    let speed = SPEED_OF_SOUND_METERS_PER_SECOND;
    let ratio = ((speed - listener_velocity) / (speed - source_velocity).max(1.0)).clamp(0.5, 2.0);
    (1.0 + (ratio - 1.0) * source.spatial.doppler_factor * MAX_DOPPLER_PREVIEW_GAIN_OFFSET)
        .clamp(0.5, 1.5)
}
