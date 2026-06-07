use std::collections::HashMap;

use zircon_runtime::core::framework::sound::{
    ExternalAudioSourceHandle, SoundExternalSourceBlock, SoundSourceDescriptor,
    SoundSourceFinishReason,
};

use crate::engine::state::SourceVoice;
use crate::SoundConfig;

use super::super::external::mix_external_source_block;

pub(super) fn mix_external_source_input(
    destination: &mut [f32],
    output_channels: usize,
    frames: usize,
    voice: &mut SourceVoice,
    descriptor: &SoundSourceDescriptor,
    handle: &ExternalAudioSourceHandle,
    external_sources: &HashMap<ExternalAudioSourceHandle, SoundExternalSourceBlock>,
    config: &SoundConfig,
) -> Option<SoundSourceFinishReason> {
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
        &config.channel_layout,
        &mut voice.cursor_frame,
        &mut voice.cursor_position,
    );
    finished.then_some(SoundSourceFinishReason::Completed)
}
