use super::validation::assert_complete_frames;
use zircon_runtime::asset::{AssetUri, SoundAsset};
use zircon_runtime::core::framework::sound::SoundChannelLayout;

pub(in crate::tests) fn test_clip(uri: &str, mono_samples: &[f32]) -> SoundAsset {
    test_clip_with_rate(uri, 48_000, mono_samples)
}

pub(in crate::tests) fn test_clip_with_rate(
    uri: &str,
    sample_rate_hz: u32,
    mono_samples: &[f32],
) -> SoundAsset {
    test_clip_with_layout(
        uri,
        sample_rate_hz,
        SoundChannelLayout::mono(),
        mono_samples,
    )
}

pub(in crate::tests) fn test_stereo_clip_with_rate(
    uri: &str,
    sample_rate_hz: u32,
    stereo_samples: &[f32],
) -> SoundAsset {
    test_clip_with_layout(
        uri,
        sample_rate_hz,
        SoundChannelLayout::stereo(),
        stereo_samples,
    )
}

pub(in crate::tests) fn test_clip_with_channels(
    uri: &str,
    sample_rate_hz: u32,
    channel_count: u16,
    samples: &[f32],
) -> SoundAsset {
    test_clip_with_layout(
        uri,
        sample_rate_hz,
        SoundChannelLayout::for_channel_count(channel_count),
        samples,
    )
}

pub(in crate::tests) fn test_clip_with_layout(
    uri: &str,
    sample_rate_hz: u32,
    channel_layout: SoundChannelLayout,
    samples: &[f32],
) -> SoundAsset {
    let channel_count = channel_layout.channel_count;
    assert_complete_frames(channel_count, samples);
    SoundAsset {
        uri: AssetUri::parse(uri).unwrap(),
        sample_rate_hz,
        channel_count,
        channel_layout,
        samples: samples.to_vec(),
    }
}
