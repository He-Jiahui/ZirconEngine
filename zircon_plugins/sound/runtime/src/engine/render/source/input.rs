mod clip;
mod external;
mod synth;

use std::collections::HashMap;

use zircon_runtime::core::framework::sound::{
    ExternalAudioSourceHandle, SoundClipId, SoundExternalSourceBlock, SoundParameterId,
    SoundSourceDescriptor, SoundSourceFinishReason, SoundSourceInput,
};

use crate::SoundConfig;

use super::super::super::state::{LoadedClip, SourceVoice};
use clip::mix_clip_source_input;
use external::mix_external_source_input;
use synth::mix_synth_parameter_source_input;

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
        SoundSourceInput::Clip(clip_id) => mix_clip_source_input(
            destination,
            output_channels,
            frames,
            voice,
            descriptor,
            *clip_id,
            clips,
            config,
        ),
        SoundSourceInput::External(handle) => mix_external_source_input(
            destination,
            output_channels,
            frames,
            voice,
            descriptor,
            handle,
            external_sources,
            config,
        ),
        SoundSourceInput::SynthParameter {
            parameter,
            default_value,
        } => mix_synth_parameter_source_input(
            destination,
            descriptor,
            parameter,
            *default_value,
            parameters,
        ),
        SoundSourceInput::Silence => None,
    }
}
