use zircon_runtime::core::framework::sound::{SoundChannelLayout, SoundSpeakerChannel};

use super::direct::speaker_sample;
use super::weights::CENTER_DOWNMIX_GAIN;

pub(super) fn phantom_center_sample_for_output(
    source_frame: &[f32],
    source_layout: &SoundChannelLayout,
    output_layout: &SoundChannelLayout,
    output_speaker: SoundSpeakerChannel,
) -> f32 {
    if output_layout
        .speakers
        .contains(&SoundSpeakerChannel::FrontCenter)
    {
        return 0.0;
    }
    if !source_layout
        .speakers
        .contains(&SoundSpeakerChannel::FrontCenter)
    {
        return 0.0;
    }

    match output_speaker {
        SoundSpeakerChannel::FrontLeft | SoundSpeakerChannel::FrontRight => {
            speaker_sample(
                source_frame,
                source_layout,
                SoundSpeakerChannel::FrontCenter,
            ) * CENTER_DOWNMIX_GAIN
        }
        _ => 0.0,
    }
}
