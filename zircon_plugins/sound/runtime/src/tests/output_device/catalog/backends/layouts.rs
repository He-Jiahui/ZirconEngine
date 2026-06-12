use super::super::super::super::*;
use super::support::software_null_backend;

#[test]
fn software_null_backend_lists_supported_channel_layouts() {
    let sound = DefaultSoundManager::default();
    let backend = software_null_backend(&sound);

    assert!(backend
        .supported_channel_layouts
        .contains(&SoundChannelLayout::stereo()));
    assert!(backend
        .supported_channel_layouts
        .contains(&SoundChannelLayout::quad()));
    assert!(backend
        .supported_channel_layouts
        .contains(&SoundChannelLayout::surround_5_1()));
    assert!(backend
        .supported_channel_layouts
        .contains(&SoundChannelLayout::surround_5_1_side()));
}
