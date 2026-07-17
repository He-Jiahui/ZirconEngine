use zircon_runtime::core::framework::audio::AudioChannelLayout;
use zircon_runtime::core::framework::sound::SoundOutputDeviceState;

use crate::kira_bridge::{device_info_for_test, KIRA_CPAL_BACKEND};
use crate::SoundConfig;

use super::support::kira_catalog_fixture;

#[test]
fn cpal_backend_catalog_reports_realtime_kira_capabilities() {
    let fixture = kira_catalog_fixture();
    let backend = fixture.backends.first().unwrap();

    assert_eq!(backend.backend, KIRA_CPAL_BACKEND);
    assert!(backend.realtime_capable);
    assert!(!backend.deterministic);
    assert!(backend.min_sample_rate_hz <= 48_000);
    assert!(backend.max_sample_rate_hz >= 48_000);
    assert!(backend
        .notes
        .iter()
        .any(|note| note.contains("Kira owns the audio thread")));
}

#[test]
fn cpal_backend_catalog_limits_m1_output_layouts_to_mono_and_stereo() {
    let fixture = kira_catalog_fixture();
    let backend = fixture.backends.first().unwrap();

    assert_eq!(
        backend.supported_channel_layouts,
        vec![AudioChannelLayout::mono(), AudioChannelLayout::stereo()]
    );
    assert_eq!(backend.min_channel_count, 1);
    assert_eq!(backend.max_channel_count, 2);
}

#[test]
fn cpal_device_catalog_projects_only_kira_cpal_descriptors() {
    let fixture = kira_catalog_fixture();

    assert_eq!(fixture.output.descriptor().backend, KIRA_CPAL_BACKEND);
    assert_eq!(
        fixture.output.descriptor().sample_rate_hz,
        fixture.config.sample_rate_hz
    );
    assert_eq!(
        fixture.output.descriptor().channel_count,
        fixture.config.channel_count
    );
    assert!(fixture.devices.iter().all(|device| {
        device.descriptor.backend == KIRA_CPAL_BACKEND
            && device.descriptor.channel_count <= 2
            && !device.descriptor.id.as_str().starts_with("software-")
    }));
}

#[test]
fn cpal_device_catalog_preserves_unsupported_multichannel_requests_without_downcasting() {
    let config = SoundConfig {
        channel_count: 6,
        channel_layout: AudioChannelLayout::surround_5_1(),
        ..SoundConfig::default()
    };

    let device = device_info_for_test(&config);

    assert_eq!(device.descriptor.channel_count, 6);
    assert_eq!(
        device.descriptor.channel_layout,
        AudioChannelLayout::surround_5_1()
    );
    assert!(!device.available);
    assert!(device
        .diagnostic
        .as_deref()
        .is_some_and(|diagnostic| diagnostic.contains("unavailable in Kira v1")));
}

#[test]
fn cpal_device_status_uses_kira_descriptor_without_retired_picker_fixture() {
    let mut fixture = kira_catalog_fixture();
    fixture.output.mark_started();

    let status = fixture.output.status();
    assert_eq!(status.descriptor.backend, KIRA_CPAL_BACKEND);
    assert_eq!(status.state, SoundOutputDeviceState::Started);
    assert_eq!(status.rendered_frames, 0);
    assert_eq!(status.latency.queued_samples, None);
    assert_eq!(status.latency.capacity_samples, None);
    assert!(status.diagnostics.is_empty());
}
