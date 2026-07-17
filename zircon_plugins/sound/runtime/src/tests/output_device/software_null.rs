use super::super::*;

#[test]
fn retired_software_null_backend_is_rejected_by_the_kira_runtime_contract() {
    let sound = DefaultSoundManager::default();
    let error = sound
        .configure_output_device(SoundOutputDeviceDescriptor {
            id: SoundOutputDeviceId::new("software-null:test"),
            backend: "software-null".to_string(),
            display_name: "Retired software-null test device".to_string(),
            sample_rate_hz: 48_000,
            channel_count: 2,
            channel_layout: AudioChannelLayout::stereo(),
            block_size_frames: 256,
            latency_blocks: 2,
        })
        .unwrap_err();

    assert!(matches!(error, SoundError::BackendUnavailable { .. }));
    assert!(error.to_string().contains("software-null"));
}
