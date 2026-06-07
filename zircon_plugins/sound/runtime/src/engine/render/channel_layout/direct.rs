use zircon_runtime::core::framework::sound::{SoundChannelLayout, SoundSpeakerChannel};

pub(in crate::engine::render) fn speaker_sample(
    source_frame: &[f32],
    source_layout: &SoundChannelLayout,
    speaker: SoundSpeakerChannel,
) -> f32 {
    source_layout
        .speakers
        .iter()
        .position(|candidate| *candidate == speaker)
        .and_then(|index| source_frame.get(index))
        .copied()
        .unwrap_or_default()
}
