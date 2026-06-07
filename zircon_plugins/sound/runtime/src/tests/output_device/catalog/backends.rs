use super::super::super::*;

#[test]
fn output_backends_list_deterministic_null_backend() {
    let sound = DefaultSoundManager::default();
    let backends = sound.available_output_backends().unwrap();
    let backend = backends
        .iter()
        .find(|backend| backend.backend == "software-null")
        .expect("software-null backend should be listed");

    assert!(backend.deterministic);
    assert!(!backend.realtime_capable);
    assert!(backend.max_sample_rate_hz >= 48_000);
    assert!(backend.max_channel_count >= 2);
    assert!(backend
        .supported_channel_layouts
        .contains(&SoundChannelLayout::stereo()));
    assert!(backend
        .supported_channel_layouts
        .contains(&SoundChannelLayout::quad()));
    assert!(backend
        .supported_channel_layouts
        .contains(&SoundChannelLayout::surround_5_1()));
    assert!(backend
        .supported_channel_layouts
        .contains(&SoundChannelLayout::surround_5_1_side()));
}
