use std::collections::HashMap;

use zircon_runtime::core::framework::sound::{
    ExternalAudioSourceHandle, SoundClipId, SoundExternalSourceBlock, SoundParameterId,
    SoundPlaybackCompletionAction, SoundSourceDescriptor, SoundSourceFinishReason,
    SoundSourceInput,
};

use crate::SoundConfig;

use super::super::super::state::{ActivePlayback, LoadedClip, SourceVoice};
use super::super::playback::mix_clip_playback;
use super::external::mix_external_source_block;
use super::range::source_clip_range;

pub(super) fn mix_source_voice(
    destination: &mut [f32],
    output_channels: usize,
    frames: usize,
    voice: &mut SourceVoice,
    descriptor: &SoundSourceDescriptor,
    clips: &HashMap<SoundClipId, LoadedClip>,
    external_sources: &HashMap<ExternalAudioSourceHandle, SoundExternalSourceBlock>,
    parameters: &HashMap<SoundParameterId, f32>,
    config: &SoundConfig,
) -> Option<SoundSourceFinishReason> {
    match &descriptor.input {
        SoundSourceInput::Clip(clip_id) => {
            let Some(clip) = clips.get(clip_id) else {
                return Some(SoundSourceFinishReason::MissingClip);
            };
            let range = source_clip_range(
                descriptor,
                clip.asset.sample_rate_hz,
                clip.asset.frame_count(),
            );
            let mut playback = ActivePlayback {
                clip: *clip_id,
                cursor_frame: voice.cursor_frame,
                cursor_position: voice.cursor_position,
                gain: descriptor.gain,
                speed: descriptor.speed,
                looped: descriptor.looped,
                completion_action: SoundPlaybackCompletionAction::None,
                paused: false,
                muted: descriptor.muted,
                range_start_frame: range.0,
                range_end_frame: range.1,
                output_track: descriptor.output_track,
                pan: 0.0,
            };
            let finished = mix_clip_playback(
                destination,
                output_channels,
                frames,
                &clip.asset,
                &mut playback,
                config,
            );
            voice.cursor_frame = playback.cursor_frame;
            voice.cursor_position = playback.cursor_position;
            finished.then_some(SoundSourceFinishReason::Completed)
        }
        SoundSourceInput::External(handle) => {
            let Some(block) = external_sources.get(handle) else {
                return None;
            };
            let finished = mix_external_source_block(
                destination,
                output_channels,
                frames,
                block,
                descriptor.gain,
                descriptor.looped,
                config.sample_rate_hz,
                &mut voice.cursor_frame,
                &mut voice.cursor_position,
            );
            finished.then_some(SoundSourceFinishReason::Completed)
        }
        SoundSourceInput::SynthParameter {
            parameter,
            default_value,
        } => {
            let value = parameters
                .get(parameter)
                .copied()
                .unwrap_or(*default_value)
                .clamp(-1.0, 1.0);
            for sample in destination {
                *sample += value * descriptor.gain;
            }
            None
        }
        SoundSourceInput::Silence => None,
    }
}
