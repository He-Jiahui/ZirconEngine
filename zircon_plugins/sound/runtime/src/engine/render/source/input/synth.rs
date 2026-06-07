use std::collections::HashMap;

use zircon_runtime::core::framework::sound::{
    SoundParameterId, SoundSourceDescriptor, SoundSourceFinishReason,
};

pub(super) fn mix_synth_parameter_source_input(
    destination: &mut [f32],
    descriptor: &SoundSourceDescriptor,
    parameter: &SoundParameterId,
    default_value: f32,
    parameters: &HashMap<SoundParameterId, f32>,
) -> Option<SoundSourceFinishReason> {
    let value = parameters
        .get(parameter)
        .copied()
        .unwrap_or(default_value)
        .clamp(-1.0, 1.0);
    for sample in destination {
        *sample += value * descriptor.gain;
    }
    None
}
