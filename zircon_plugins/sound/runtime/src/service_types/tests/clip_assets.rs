use zircon_runtime::core::framework::audio::AudioChannelLayout;

use super::*;

#[test]
fn cached_clip_load_returns_before_project_asset_resolution() {
    let manager = DefaultSoundManager::default();
    let locator = "res://audio/cached.wav";
    let clip = manager.insert_clip_for_test(SoundAsset {
        uri: AssetUri::parse(locator).unwrap(),
        sample_rate_hz: 48_000,
        channel_count: 1,
        channel_layout: AudioChannelLayout::mono(),
        samples: vec![0.0],
    });
    crate::poison_recovery::lock_recover(&manager.state)
        .clip_ids_by_locator
        .insert(locator.to_string(), clip);

    assert_eq!(manager.load_clip_impl(locator).unwrap(), clip);
}
