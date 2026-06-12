use super::super::super::super::*;
use super::support::software_null_picker;

#[test]
fn output_devices_list_software_null_picker_descriptor() {
    let sound = DefaultSoundManager::default();
    let software = software_null_picker(&sound);

    assert!(software.is_default);
    assert!(software.available);
    assert_eq!(software.diagnostic, None);
    assert_eq!(software.descriptor.display_name, "Software Output");
    assert_eq!(
        software.descriptor.channel_layout,
        SoundChannelLayout::stereo()
    );
}
