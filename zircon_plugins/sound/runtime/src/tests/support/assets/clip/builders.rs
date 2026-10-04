// 测试音频资源统一经完整帧检查与显式布局构造，供播放、Kira 和空间声用例复用。
use super::validation::assert_complete_frames;
use zircon_runtime::asset::{AssetUri, SoundAsset};
use zircon_runtime::core::framework::audio::AudioChannelLayout;

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
        AudioChannelLayout::mono(),
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
        AudioChannelLayout::stereo(),
        stereo_samples,
    )
}

// 数量版本用框架默认布局；需测试非默认扬声器顺序时应调用显式布局版本。
pub(in crate::tests) fn test_clip_with_channels(
    uri: &str,
    sample_rate_hz: u32,
    channel_count: u16,
    samples: &[f32],
) -> SoundAsset {
    test_clip_with_layout(
        uri,
        sample_rate_hz,
        AudioChannelLayout::for_channel_count(channel_count),
        samples,
    )
}

// 夹具入口仅保证非零声道和完整帧；无效采样率或布局语义仍可由调用方构造给生产校验测试。
pub(in crate::tests) fn test_clip_with_layout(
    uri: &str,
    sample_rate_hz: u32,
    channel_layout: AudioChannelLayout,
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
