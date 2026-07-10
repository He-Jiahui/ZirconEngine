use zircon_runtime::core::framework::audio::{AudioChannelLayout, AudioSpeakerChannel};

use super::direct::speaker_sample;
use super::weights::{CENTER_DOWNMIX_GAIN, REAR_DOWNMIX_GAIN, SIDE_DOWNMIX_GAIN};

pub(super) fn stereo_downmix_sample(
    source_frame: &[f32],
    source_layout: &AudioChannelLayout,
    output_speaker: AudioSpeakerChannel,
) -> f32 {
    match output_speaker {
        AudioSpeakerChannel::FrontLeft => {
            speaker_sample(source_frame, source_layout, AudioSpeakerChannel::FrontLeft)
                + speaker_sample(
                    source_frame,
                    source_layout,
                    AudioSpeakerChannel::FrontCenter,
                ) * CENTER_DOWNMIX_GAIN
                + speaker_sample(source_frame, source_layout, AudioSpeakerChannel::SideLeft)
                    * SIDE_DOWNMIX_GAIN
                + speaker_sample(source_frame, source_layout, AudioSpeakerChannel::BackLeft)
                    * REAR_DOWNMIX_GAIN
        }
        AudioSpeakerChannel::FrontRight => {
            speaker_sample(source_frame, source_layout, AudioSpeakerChannel::FrontRight)
                + speaker_sample(
                    source_frame,
                    source_layout,
                    AudioSpeakerChannel::FrontCenter,
                ) * CENTER_DOWNMIX_GAIN
                + speaker_sample(source_frame, source_layout, AudioSpeakerChannel::SideRight)
                    * SIDE_DOWNMIX_GAIN
                + speaker_sample(source_frame, source_layout, AudioSpeakerChannel::BackRight)
                    * REAR_DOWNMIX_GAIN
        }
        speaker => speaker_sample(source_frame, source_layout, speaker),
    }
}

pub(super) fn uses_front_pair_downmix(output_layout: &AudioChannelLayout) -> bool {
    output_layout.speakers.len() == 2
        && output_layout
            .speakers
            .contains(&AudioSpeakerChannel::FrontLeft)
        && output_layout
            .speakers
            .contains(&AudioSpeakerChannel::FrontRight)
}
