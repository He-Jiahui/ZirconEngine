use zircon_runtime::asset::{AssetUri, SoundAsset};

pub(in crate::tests) fn test_clip(uri: &str, mono_samples: &[f32]) -> SoundAsset {
    test_clip_with_rate(uri, 48_000, mono_samples)
}

pub(in crate::tests) fn test_clip_with_rate(
    uri: &str,
    sample_rate_hz: u32,
    mono_samples: &[f32],
) -> SoundAsset {
    SoundAsset {
        uri: AssetUri::parse(uri).unwrap(),
        sample_rate_hz,
        channel_count: 1,
        samples: mono_samples.to_vec(),
    }
}
