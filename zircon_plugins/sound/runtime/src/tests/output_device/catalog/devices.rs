use super::super::super::*;

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
    assert_eq!(
        software.descriptor.channel_layout,
        SoundChannelLayout::stereo()
    );

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
