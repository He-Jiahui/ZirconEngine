use zircon_runtime::core::framework::audio::{AudioChannelLayout, AudioSpeakerChannel};

use super::direct::speaker_sample;
use super::weights::CENTER_DOWNMIX_GAIN;

pub(super) fn phantom_center_sample_for_output(
    source_frame: &[f32],
    source_layout: &AudioChannelLayout,
    output_layout: &AudioChannelLayout,
    output_speaker: AudioSpeakerChannel,
) -> f32 {
    if output_layout
        .speakers
        .contains(&AudioSpeakerChannel::FrontCenter)
    {
        return 0.0;
    }
    if !source_layout
        .speakers
        .contains(&AudioSpeakerChannel::FrontCenter)
    {
        return 0.0;
    }

    match output_speaker {
        AudioSpeakerChannel::FrontLeft | AudioSpeakerChannel::FrontRight => {
            speaker_sample(
                source_frame,
                source_layout,
                AudioSpeakerChannel::FrontCenter,
            ) * CENTER_DOWNMIX_GAIN
        }
        _ => 0.0,
    }
}
