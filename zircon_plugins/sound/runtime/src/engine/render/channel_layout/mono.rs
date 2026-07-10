use zircon_runtime::core::framework::audio::{AudioChannelLayout, AudioSpeakerChannel};

use super::stereo::stereo_downmix_sample;
use super::weights::STEREO_TO_MONO_GAIN;

pub(super) fn mono_source_sample_for_output(
    sample: f32,
    output_layout: &AudioChannelLayout,
    output_channel: usize,
) -> f32 {
    let Some(output_speaker) = output_layout.speakers.get(output_channel).copied() else {
        return sample;
    };
    let has_front_center = output_layout
        .speakers
        .contains(&AudioSpeakerChannel::FrontCenter);
    if has_front_center {
        (output_speaker == AudioSpeakerChannel::FrontCenter)
            .then_some(sample)
            .unwrap_or_default()
    } else {
        matches!(
            output_speaker,
            AudioSpeakerChannel::FrontLeft | AudioSpeakerChannel::FrontRight
        )
        .then_some(sample)
        .unwrap_or_default()
    }
}

pub(super) fn mono_downmix(source_frame: &[f32], source_layout: &AudioChannelLayout) -> f32 {
    if let [sample] = source_frame {
        return *sample;
    }
    if source_layout.speakers.is_empty() {
        return average(source_frame);
    }
    stereo_fold_down(source_frame, source_layout) * STEREO_TO_MONO_GAIN
}

fn stereo_fold_down(source_frame: &[f32], source_layout: &AudioChannelLayout) -> f32 {
    stereo_downmix_sample(source_frame, source_layout, AudioSpeakerChannel::FrontLeft)
        + stereo_downmix_sample(source_frame, source_layout, AudioSpeakerChannel::FrontRight)
}

fn average(samples: &[f32]) -> f32 {
    samples.iter().copied().sum::<f32>() / samples.len() as f32
}
