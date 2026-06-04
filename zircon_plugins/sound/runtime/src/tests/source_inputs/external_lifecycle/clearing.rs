use super::super::super::*;

#[test]
fn external_audio_source_clear_reports_unknown_and_clears_existing_blocks() {
    let sound = DefaultSoundManager::default();
    let handle = ExternalAudioSourceHandle::new("navigation.surface-noise");

    assert!(matches!(
        sound.clear_external_source(&handle).unwrap_err(),
        SoundError::UnknownExternalSource { .. }
    ));

    sound
        .submit_external_source_block(
            handle.clone(),
            SoundExternalSourceBlock {
                sample_rate_hz: 48_000,
                channel_count: 1,
                samples: vec![0.75],
            },
        )
        .unwrap();
    sound.clear_external_source(&handle).unwrap();
}
