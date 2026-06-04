use super::super::super::super::*;

#[test]
fn software_null_backend_reports_unsupported_backend_and_recovers() {
    let sound = DefaultSoundManager::default();

    let error = sound
        .configure_output_device(SoundOutputDeviceDescriptor {
            id: SoundOutputDeviceId::new("sound.output.unsupported"),
            backend: "native-missing".to_string(),
            display_name: "Unsupported Native".to_string(),
            sample_rate_hz: 48_000,
            channel_count: 2,
            channel_layout: SoundChannelLayout::stereo(),
            block_size_frames: 128,
            latency_blocks: 2,
        })
        .unwrap_err();
    assert!(error.to_string().contains("not available"));

    let status = sound.backend_status();
    assert_eq!(status.requested_backend, "native-missing");
    assert_eq!(status.active_backend, None);
    assert_eq!(status.state, SoundBackendState::Unavailable);
    assert!(status
        .detail
        .as_deref()
        .unwrap_or_default()
        .contains("not available"));
    assert_eq!(sound.backend_name(), "native-missing");
    assert!(sound
        .start_output_device()
        .unwrap_err()
        .to_string()
        .contains("not available"));
    assert!(sound
        .pull_output_backend_callback()
        .unwrap_err()
        .to_string()
        .contains("not available"));
    assert_eq!(
        sound.output_device_status().unwrap().state,
        SoundOutputDeviceState::Stopped
    );
    assert!(sound
        .output_device_status()
        .unwrap()
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("not available")));

    sound
        .configure_output_device(SoundOutputDeviceDescriptor {
            id: SoundOutputDeviceId::new("sound.output.null.retry"),
            backend: "software-null".to_string(),
            display_name: "Software Null Retry Output".to_string(),
            sample_rate_hz: 48_000,
            channel_count: 2,
            channel_layout: SoundChannelLayout::stereo(),
            block_size_frames: 128,
            latency_blocks: 2,
        })
        .unwrap();

    let status = sound.backend_status();
    assert_eq!(status.requested_backend, "software-null");
    assert_eq!(status.active_backend.as_deref(), Some("software-null"));
    assert_eq!(status.state, SoundBackendState::Ready);
    assert_eq!(status.detail, None);
}
