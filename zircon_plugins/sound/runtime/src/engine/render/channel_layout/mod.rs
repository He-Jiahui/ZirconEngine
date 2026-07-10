mod center;
mod direct;
mod discrete;
mod downmix;
mod mono;
mod positional;
mod stereo;
mod weights;

use zircon_runtime::core::framework::audio::AudioChannelLayout;

use discrete::discrete_source_sample_for_output;
use downmix::named_source_sample_for_output;
use mono::{mono_downmix, mono_source_sample_for_output};

pub(in crate::engine::render) fn source_frame_sample_for_output(
    source_frame: &[f32],
    source_layout: &AudioChannelLayout,
    output_layout: &AudioChannelLayout,
    output_channel: usize,
) -> f32 {
    if source_frame.is_empty() {
        return 0.0;
    }
    if output_layout.channel_count == 1 {
        return mono_downmix(source_frame, source_layout);
    }
    if source_frame.len() == 1 {
        return mono_source_sample_for_output(source_frame[0], output_layout, output_channel);
    }
    if source_layout.speakers.is_empty() {
        return discrete_source_sample_for_output(source_frame, output_layout, output_channel);
    }
    let Some(output_speaker) = output_layout.speakers.get(output_channel).copied() else {
        return discrete_source_sample_for_output(source_frame, output_layout, output_channel);
    };

    named_source_sample_for_output(source_frame, source_layout, output_layout, output_speaker)
}
