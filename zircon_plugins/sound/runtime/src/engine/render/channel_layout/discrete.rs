use zircon_runtime::core::framework::audio::{AudioChannelLayout, AudioSpeakerChannel};

use super::positional::positional_source_sample;
use super::weights::DISCRETE_OVERFLOW_DOWNMIX_GAIN;

pub(super) fn discrete_source_sample_for_output(
    source_frame: &[f32],
    output_layout: &AudioChannelLayout,
    output_channel: usize,
) -> f32 {
    if output_layout.speakers.is_empty() {
        return positional_source_sample(source_frame, output_channel);
    }

    let direct_sample = positional_source_sample(source_frame, output_channel);
    match output_layout.speakers.get(output_channel).copied() {
        Some(AudioSpeakerChannel::FrontLeft) => {
            direct_sample + stereo_overflow_sample(source_frame, output_layout, 0)
        }
        Some(AudioSpeakerChannel::FrontRight) => {
            direct_sample + stereo_overflow_sample(source_frame, output_layout, 1)
        }
        _ => direct_sample,
    }
}

fn stereo_overflow_sample(
    source_frame: &[f32],
    output_layout: &AudioChannelLayout,
    pair_channel: usize,
) -> f32 {
    source_frame
        .iter()
        .copied()
        .enumerate()
        .skip(usize::from(output_layout.channel_count))
        .filter(|(source_channel, _)| source_channel % 2 == pair_channel)
        .map(|(_, sample)| sample * DISCRETE_OVERFLOW_DOWNMIX_GAIN)
        .sum()
}
