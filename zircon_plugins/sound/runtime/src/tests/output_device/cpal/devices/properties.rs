use super::super::super::super::*;
use super::support::cpal_output_devices;

#[test]
fn cpal_output_devices_project_required_descriptor_properties() {
    let sound = DefaultSoundManager::default();
    let cpal_devices = cpal_output_devices(&sound);

    for device in cpal_devices {
        assert_eq!(device.descriptor.backend, "cpal");
        assert!(!device.descriptor.display_name.trim().is_empty());
        assert!(device.descriptor.sample_rate_hz > 0);
        assert!(device.descriptor.channel_count > 0);
        assert!(device.descriptor.block_size_frames > 0);
        assert!(device.descriptor.latency_blocks > 0);
    }
}
