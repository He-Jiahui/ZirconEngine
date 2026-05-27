use super::*;

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
}

#[test]
fn output_devices_list_configurable_software_null_picker_descriptor() {
    let sound = DefaultSoundManager::default();
    let devices = sound.available_output_devices().unwrap();
    let software = devices
        .iter()
        .find(|device| device.descriptor.backend == "software-null")
        .expect("software-null output device should be listed");

    assert!(software.is_default);
    assert!(software.available);
    assert_eq!(software.diagnostic, None);
    assert_eq!(software.descriptor.display_name, "Software Output");

    sound
        .configure_output_device(software.descriptor.clone())
        .unwrap();
    let status = sound.output_device_status().unwrap();
    assert_eq!(status.descriptor, software.descriptor);
    assert_eq!(status.latency.requested_latency_blocks, 2);
    assert_eq!(
        status.latency.estimated_latency_frames,
        status.descriptor.block_size_frames * status.descriptor.latency_blocks
    );
    assert!(status.latency.estimated_latency_seconds > 0.0);
    assert_eq!(status.latency.queued_samples, None);
    assert_eq!(status.latency.capacity_samples, None);
    assert!(status.diagnostics.is_empty());
}

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
            block_size_frames: 128,
            latency_blocks: 2,
        })
        .unwrap();
    assert_eq!(sound.backend_status().state, SoundBackendState::Ready);
}

#[cfg(feature = "cpal-backend")]
#[test]
fn cpal_backend_is_listed_when_feature_is_enabled() {
    let sound = DefaultSoundManager::default();
    let backend = sound
        .available_output_backends()
        .unwrap()
        .into_iter()
        .find(|backend| backend.backend == "cpal")
        .expect("cpal backend should be listed with cpal-backend feature");
    assert!(backend.realtime_capable);
    assert!(!backend.deterministic);
}

#[cfg(feature = "cpal-backend")]
#[test]
fn cpal_output_device_enumeration_is_structured_when_feature_is_enabled() {
    let sound = DefaultSoundManager::default();
    let devices = sound.available_output_devices().unwrap();
    let cpal_devices = devices
        .iter()
        .filter(|device| device.descriptor.backend == "cpal")
        .collect::<Vec<_>>();

    assert!(!cpal_devices.is_empty());
    assert!(cpal_devices
        .iter()
        .any(|device| device.descriptor.id.as_str() == "sound.output.cpal.default"));
    for device in cpal_devices {
        assert_eq!(device.descriptor.backend, "cpal");
        assert!(!device.descriptor.display_name.trim().is_empty());
        assert!(device.descriptor.sample_rate_hz > 0);
        assert!(device.descriptor.channel_count > 0);
        assert!(device.descriptor.block_size_frames > 0);
        assert!(device.descriptor.latency_blocks > 0);
    }
}

#[test]
fn software_null_backend_callback_reports_rendered_block() {
    let sound = DefaultSoundManager::default();
    sound
        .configure_output_device(SoundOutputDeviceDescriptor {
            id: SoundOutputDeviceId::new("sound.output.null"),
            backend: "software-null".to_string(),
            display_name: "Software Null Output".to_string(),
            sample_rate_hz: 48_000,
            channel_count: 2,
            block_size_frames: 2,
            latency_blocks: 2,
        })
        .unwrap();
    sound.start_output_device().unwrap();

    let clip = sound.insert_clip_for_test(test_clip("res://sound/null-output.wav", &[0.25, 0.5]));
    sound
        .play_clip(clip, SoundPlaybackSettings::default())
        .unwrap();

    let callback = sound.pull_output_backend_callback().unwrap();
    assert_eq!(callback.report.backend, "software-null");
    assert_eq!(callback.report.sequence_index, 0);
    assert_eq!(callback.report.requested_frames, 2);
    assert_eq!(callback.report.rendered_frames, 2);
    assert_eq!(callback.report.sample_count, 4);
    assert!(!callback.report.underrun);
    assert_eq!(callback.report.error, None);
    assert_samples_near(&callback.block.samples, &[0.25, 0.25, 0.5, 0.5]);

    let status = sound.output_device_status().unwrap();
    assert_eq!(status.callback_count, 1);
    assert_eq!(status.last_callback_sequence, Some(0));
    assert_eq!(status.rendered_blocks, 1);
    assert_eq!(status.rendered_frames, 2);
    assert_eq!(status.latency.requested_latency_blocks, 2);
    assert_eq!(status.latency.estimated_latency_frames, 4);
    assert!(status.diagnostics.is_empty());
}

#[test]
fn software_null_backend_rejects_stopped_callback_and_unsupported_backend() {
    let sound = DefaultSoundManager::default();
    assert!(sound
        .pull_output_backend_callback()
        .unwrap_err()
        .to_string()
        .contains("stopped"));

    let error = sound
        .configure_output_device(SoundOutputDeviceDescriptor {
            id: SoundOutputDeviceId::new("sound.output.unsupported"),
            backend: "native-missing".to_string(),
            display_name: "Unsupported Native".to_string(),
            sample_rate_hz: 48_000,
            channel_count: 2,
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

#[cfg(all(feature = "cpal-backend", target_os = "windows"))]
#[test]
fn cpal_backend_start_stop_is_structured_on_windows() {
    let sound = DefaultSoundManager::default();
    sound
        .configure_output_device(SoundOutputDeviceDescriptor {
            id: SoundOutputDeviceId::new("sound.output.cpal.windows"),
            backend: "cpal".to_string(),
            display_name: "CPAL Windows Default Output".to_string(),
            sample_rate_hz: 48_000,
            channel_count: 2,
            block_size_frames: 128,
            latency_blocks: 2,
        })
        .unwrap();

    match sound.start_output_device() {
        Ok(()) => {
            assert_eq!(
                sound.output_device_status().unwrap().state,
                SoundOutputDeviceState::Started
            );
            sound.stop_output_device().unwrap();
            assert_eq!(
                sound.output_device_status().unwrap().state,
                SoundOutputDeviceState::Stopped
            );
        }
        Err(error) => {
            assert!(error.to_string().contains("cpal") || error.to_string().contains("device"));
            assert_eq!(
                sound.output_device_status().unwrap().state,
                SoundOutputDeviceState::Stopped
            );
        }
    }
}

#[test]
fn output_device_can_be_configured_started_and_pulled() {
    let sound = DefaultSoundManager::default();
    let descriptor = SoundOutputDeviceDescriptor {
        id: SoundOutputDeviceId::new("sound.output.test"),
        backend: "software-test".to_string(),
        display_name: "Software Test Output".to_string(),
        sample_rate_hz: 48_000,
        channel_count: 2,
        block_size_frames: 2,
        latency_blocks: 2,
    };
    sound.configure_output_device(descriptor.clone()).unwrap();
    sound.start_output_device().unwrap();

    let clip = sound.insert_clip_for_test(test_clip("res://sound/output.wav", &[0.25, 0.5]));
    sound
        .play_clip(clip, SoundPlaybackSettings::default())
        .unwrap();

    let block = sound.render_output_device_block().unwrap();
    assert_samples_near(&block.samples, &[0.25, 0.25, 0.5, 0.5]);

    let status = sound.output_device_status().unwrap();
    assert_eq!(status.descriptor, descriptor);
    assert_eq!(status.state, SoundOutputDeviceState::Started);
    assert_eq!(status.rendered_blocks, 1);
    assert_eq!(status.rendered_frames, 2);
    assert_eq!(status.underrun_count, 0);
    assert_eq!(status.last_error, None);
}

#[test]
fn output_device_updates_runtime_format_and_stops_on_reconfigure() {
    let sound = DefaultSoundManager::default();
    sound.start_output_device().unwrap();
    sound
        .configure_output_device(SoundOutputDeviceDescriptor {
            id: SoundOutputDeviceId::new("sound.output.preview"),
            backend: "software-preview".to_string(),
            display_name: "Preview Output".to_string(),
            sample_rate_hz: 24_000,
            channel_count: 1,
            block_size_frames: 3,
            latency_blocks: 1,
        })
        .unwrap();

    let status = sound.output_device_status().unwrap();
    assert_eq!(status.state, SoundOutputDeviceState::Stopped);
    assert_eq!(status.rendered_frames, 0);
    assert_eq!(sound.backend_status().sample_rate_hz, 24_000);
    assert_eq!(sound.backend_status().channel_count, 1);
    let snapshot = sound.mixer_snapshot().unwrap();
    assert_eq!(snapshot.graph.sample_rate_hz, 24_000);
    assert_eq!(snapshot.graph.channel_count, 1);
}

#[test]
fn output_device_rejects_invalid_descriptor_and_stopped_pull() {
    let sound = DefaultSoundManager::default();
    assert!(sound
        .render_output_device_block()
        .unwrap_err()
        .to_string()
        .contains("output device is stopped"));

    let error = sound
        .configure_output_device(SoundOutputDeviceDescriptor {
            id: SoundOutputDeviceId::new("sound.output.bad"),
            backend: "software-test".to_string(),
            display_name: "Bad Output".to_string(),
            sample_rate_hz: 48_000,
            channel_count: 2,
            block_size_frames: 0,
            latency_blocks: 2,
        })
        .unwrap_err();
    assert!(error.to_string().contains("block size"));
}
