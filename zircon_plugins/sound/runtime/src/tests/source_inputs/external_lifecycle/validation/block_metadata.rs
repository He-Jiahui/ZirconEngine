use super::super::super::super::*;

#[test]
fn external_audio_source_block_rejects_channel_count_and_layout_mismatches() {
    let sound = DefaultSoundManager::default();
    let handle = ExternalAudioSourceHandle::new("navigation.surface-noise");

    assert!(sound
        .submit_external_source_block(
            handle.clone(),
            SoundExternalSourceBlock {
                sample_rate_hz: 48_000,
                channel_count: 0,
                channel_layout: AudioChannelLayout::mono(),
                samples: vec![0.0],
            },
        )
        .unwrap_err()
        .to_string()
        .contains("channel count"));
    assert!(sound
        .submit_external_source_block(
            handle,
            SoundExternalSourceBlock {
                sample_rate_hz: 48_000,
                channel_count: 2,
                channel_layout: AudioChannelLayout::mono(),
                samples: vec![0.0, 0.0],
            },
        )
        .unwrap_err()
        .to_string()
        .contains("channel layout"));
}
