use super::super::super::*;
use super::support::{
    invalid_block_size_descriptor, invalid_channel_layout_descriptor,
    invalid_speaker_metadata_descriptor,
};

#[test]
fn output_device_rejects_invalid_descriptors() {
    let sound = DefaultSoundManager::default();

    let error = sound
        .configure_output_device(invalid_block_size_descriptor())
        .unwrap_err();
    assert!(error.to_string().contains("block size"));

    let error = sound
        .configure_output_device(invalid_channel_layout_descriptor())
        .unwrap_err();
    assert!(error.to_string().contains("channel layout"));

    let error = sound
        .configure_output_device(invalid_speaker_metadata_descriptor())
        .unwrap_err();
    assert!(error.to_string().contains("canonical speaker metadata"));
}
