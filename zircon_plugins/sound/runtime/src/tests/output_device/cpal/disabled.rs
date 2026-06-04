use super::super::super::*;

#[cfg(not(feature = "cpal-backend"))]
#[test]
fn cpal_backend_reports_feature_disabled_when_not_compiled() {
    let sound = DefaultSoundManager::default();
    assert!(sound
        .available_output_backends()
        .unwrap()
        .iter()
        .all(|backend| backend.backend != "cpal"));
    assert!(sound
        .available_output_devices()
        .unwrap()
        .iter()
        .all(|device| device.descriptor.backend != "cpal"));

    let error = sound
        .configure_output_device(SoundOutputDeviceDescriptor {
            id: SoundOutputDeviceId::new("sound.output.cpal.disabled"),
            backend: "cpal".to_string(),
            display_name: "CPAL Disabled".to_string(),
            sample_rate_hz: 48_000,
            channel_count: 2,
            channel_layout: SoundChannelLayout::stereo(),
            block_size_frames: 128,
            latency_blocks: 2,
        })
        .unwrap_err();
    assert!(error.to_string().contains("cpal-backend"));
    assert_eq!(sound.backend_status().requested_backend, "cpal");
    assert_eq!(sound.backend_status().state, SoundBackendState::Unavailable);

    sound
        .configure_output_device(SoundOutputDeviceDescriptor {
            id: SoundOutputDeviceId::new("sound.output.cpal.recovery"),
            backend: "software-null".to_string(),
            display_name: "Software Null Recovery".to_string(),
            sample_rate_hz: 48_000,
            channel_count: 2,
            channel_layout: SoundChannelLayout::stereo(),
            block_size_frames: 128,
            latency_blocks: 2,
        })
        .unwrap();
    assert_eq!(sound.backend_status().state, SoundBackendState::Ready);
}
