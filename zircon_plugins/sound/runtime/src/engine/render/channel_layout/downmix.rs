use zircon_runtime::core::framework::sound::{SoundChannelLayout, SoundSpeakerChannel};

use super::center::phantom_center_sample_for_output;
use super::direct::speaker_sample;
use super::stereo::{stereo_downmix_sample, uses_front_pair_downmix};

pub(super) fn named_source_sample_for_output(
    source_frame: &[f32],
    source_layout: &SoundChannelLayout,
    output_layout: &SoundChannelLayout,
    output_speaker: SoundSpeakerChannel,
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
    source_layout: &SoundChannelLayout,
    output_layout: &SoundChannelLayout,
    output_speaker: SoundSpeakerChannel,
) -> f32 {
    if output_layout
        .speakers
        .contains(&fallback_source_speaker(output_speaker))
    {
        return 0.0;
    }

    match output_speaker {
        SoundSpeakerChannel::BackLeft => {
            speaker_sample(source_frame, source_layout, SoundSpeakerChannel::SideLeft)
        }
        SoundSpeakerChannel::BackRight => {
            speaker_sample(source_frame, source_layout, SoundSpeakerChannel::SideRight)
        }
        SoundSpeakerChannel::SideLeft => {
            speaker_sample(source_frame, source_layout, SoundSpeakerChannel::BackLeft)
        }
        SoundSpeakerChannel::SideRight => {
            speaker_sample(source_frame, source_layout, SoundSpeakerChannel::BackRight)
        }
        _ => 0.0,
    }
}

fn fallback_source_speaker(output_speaker: SoundSpeakerChannel) -> SoundSpeakerChannel {
    match output_speaker {
        SoundSpeakerChannel::BackLeft => SoundSpeakerChannel::SideLeft,
        SoundSpeakerChannel::BackRight => SoundSpeakerChannel::SideRight,
        SoundSpeakerChannel::SideLeft => SoundSpeakerChannel::BackLeft,
        SoundSpeakerChannel::SideRight => SoundSpeakerChannel::BackRight,
        speaker => speaker,
    }
}
