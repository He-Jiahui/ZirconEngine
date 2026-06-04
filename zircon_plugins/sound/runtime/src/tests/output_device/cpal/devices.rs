use super::super::super::*;

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
