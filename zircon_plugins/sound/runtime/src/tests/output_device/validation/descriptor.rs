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

#[test]
fn output_device_rejects_multichannel_layouts_without_silently_downcasting() {
    let sound = DefaultSoundManager::default();
    sound.mark_output_device_started_for_test();
    let before = sound.output_device_status().unwrap();
    let error = sound
        .configure_output_device(SoundOutputDeviceDescriptor {
            id: SoundOutputDeviceId::new("kira-cpal:surround"),
            backend: "kira-cpal".to_string(),
            display_name: "Unsupported surround output".to_string(),
            sample_rate_hz: 48_000,
            channel_count: 6,
            channel_layout: AudioChannelLayout::surround_5_1(),
            block_size_frames: 256,
            latency_blocks: 2,
        })
        .unwrap_err();
    let after = sound.output_device_status().unwrap();

    assert!(matches!(error, SoundError::UnsupportedAdvancedFeature(_)));
    assert_eq!(
        after, before,
        "invalid configuration must preserve the active device state"
    );
}
