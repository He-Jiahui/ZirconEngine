use zircon_runtime::core::framework::audio::{AudioChannelLayout, AudioSpeakerChannel};

use super::center::phantom_center_sample_for_output;
use super::direct::speaker_sample;
use super::stereo::{stereo_downmix_sample, uses_front_pair_downmix};

pub(super) fn named_source_sample_for_output(
    source_frame: &[f32],
    source_layout: &AudioChannelLayout,
    output_layout: &AudioChannelLayout,
    output_speaker: AudioSpeakerChannel,
) -> f32 {
    if uses_front_pair_downmix(output_layout) {
        return stereo_downmix_sample(source_frame, source_layout, output_speaker);
    }

    speaker_sample(source_frame, source_layout, output_speaker)
        + phantom_center_sample_for_output(
            source_frame,
            source_layout,
            output_layout,
            output_speaker,
        )
        + surround_bed_fallback_sample(source_frame, source_layout, output_layout, output_speaker)
}

fn surround_bed_fallback_sample(
    source_frame: &[f32],
    source_layout: &AudioChannelLayout,
    output_layout: &AudioChannelLayout,
    output_speaker: AudioSpeakerChannel,
) -> f32 {
    if output_layout
        .speakers
        .contains(&fallback_source_speaker(output_speaker))
    {
        return 0.0;
    }

    match output_speaker {
        AudioSpeakerChannel::BackLeft => {
            speaker_sample(source_frame, source_layout, AudioSpeakerChannel::SideLeft)
        }
        AudioSpeakerChannel::BackRight => {
            speaker_sample(source_frame, source_layout, AudioSpeakerChannel::SideRight)
        }
        AudioSpeakerChannel::SideLeft => {
            speaker_sample(source_frame, source_layout, AudioSpeakerChannel::BackLeft)
        }
        AudioSpeakerChannel::SideRight => {
            speaker_sample(source_frame, source_layout, AudioSpeakerChannel::BackRight)
        }
        _ => 0.0,
    }
}

fn fallback_source_speaker(output_speaker: AudioSpeakerChannel) -> AudioSpeakerChannel {
    match output_speaker {
        AudioSpeakerChannel::BackLeft => AudioSpeakerChannel::SideLeft,
        AudioSpeakerChannel::BackRight => AudioSpeakerChannel::SideRight,
        AudioSpeakerChannel::SideLeft => AudioSpeakerChannel::BackLeft,
        AudioSpeakerChannel::SideRight => AudioSpeakerChannel::BackRight,
        speaker => speaker,
    }
}
