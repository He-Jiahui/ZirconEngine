use zircon_runtime::core::framework::sound::{SoundChannelLayout, SoundSpeakerChannel};

use super::direct::speaker_sample;
use super::weights::{CENTER_DOWNMIX_GAIN, REAR_DOWNMIX_GAIN, SIDE_DOWNMIX_GAIN};

pub(super) fn stereo_downmix_sample(
    source_frame: &[f32],
    source_layout: &SoundChannelLayout,
    output_speaker: SoundSpeakerChannel,
) -> f32 {
    match output_speaker {
        SoundSpeakerChannel::FrontLeft => {
            speaker_sample(source_frame, source_layout, SoundSpeakerChannel::FrontLeft)
                + speaker_sample(
                    source_frame,
                    source_layout,
                    SoundSpeakerChannel::FrontCenter,
                ) * CENTER_DOWNMIX_GAIN
                + speaker_sample(source_frame, source_layout, SoundSpeakerChannel::SideLeft)
                    * SIDE_DOWNMIX_GAIN
                + speaker_sample(source_frame, source_layout, SoundSpeakerChannel::BackLeft)
                    * REAR_DOWNMIX_GAIN
        }
        SoundSpeakerChannel::FrontRight => {
            speaker_sample(source_frame, source_layout, SoundSpeakerChannel::FrontRight)
                + speaker_sample(
                    source_frame,
                    source_layout,
                    SoundSpeakerChannel::FrontCenter,
                ) * CENTER_DOWNMIX_GAIN
                + speaker_sample(source_frame, source_layout, SoundSpeakerChannel::SideRight)
                    * SIDE_DOWNMIX_GAIN
                + speaker_sample(source_frame, source_layout, SoundSpeakerChannel::BackRight)
                    * REAR_DOWNMIX_GAIN
        }
        speaker => speaker_sample(source_frame, source_layout, speaker),
    }
}

pub(super) fn uses_front_pair_downmix(output_layout: &SoundChannelLayout) -> bool {
    output_layout.speakers.len() == 2
        && output_layout
            .speakers
            .contains(&SoundSpeakerChannel::FrontLeft)
        && output_layout
            .speakers
            .contains(&SoundSpeakerChannel::FrontRight)
}
