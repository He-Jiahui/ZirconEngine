use zircon_runtime::core::framework::audio::{AudioChannelLayout, AudioSpeakerChannel};

pub(in crate::engine::render) fn speaker_sample(
    source_frame: &[f32],
    source_layout: &AudioChannelLayout,
    speaker: AudioSpeakerChannel,
) -> f32 {
    source_layout
        .speakers
        .iter()
        .position(|candidate| *candidate == speaker)
        .and_then(|index| source_frame.get(index))
        .copied()
        .unwrap_or_default()
}
