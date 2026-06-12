use super::super::super::super::*;
use super::support::cpal_output_devices;

#[test]
fn cpal_output_devices_include_default_device_when_feature_is_enabled() {
    let sound = DefaultSoundManager::default();
    let cpal_devices = cpal_output_devices(&sound);

    assert!(!cpal_devices.is_empty());
    assert!(cpal_devices
        .iter()
        .any(|device| device.descriptor.id.as_str() == "sound.output.cpal.default"));
}
