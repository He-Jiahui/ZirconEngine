use std::collections::HashMap;

use zircon_runtime::core::framework::sound::{
    SoundClipId, SoundPlaybackCompletionAction, SoundSourceDescriptor, SoundSourceFinishReason,
};

use crate::engine::state::{ActivePlayback, LoadedClip, SourceVoice};
use crate::SoundConfig;

use super::super::super::playback::mix_clip_playback;
use super::super::range::source_clip_range;

pub(super) fn mix_clip_source_input(
    destination: &mut [f32],
    output_channels: usize,
    frames: usize,
    voice: &mut SourceVoice,
    descriptor: &SoundSourceDescriptor,
    clip_id: SoundClipId,
    clips: &HashMap<SoundClipId, LoadedClip>,
    config: &SoundConfig,
) -> Option<SoundSourceFinishReason> {
    let Some(clip) = clips.get(&clip_id) else {
        return Some(SoundSourceFinishReason::MissingClip);
    };
    let range = source_clip_range(
        descriptor,
        clip.asset.sample_rate_hz,
        clip.asset.frame_count(),
    );
    let mut playback = ActivePlayback {
        clip: clip_id,
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
