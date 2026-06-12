use super::super::super::super::*;

#[test]
fn external_audio_source_block_rejects_blank_handle() {
    let sound = DefaultSoundManager::default();
    let empty_handle = ExternalAudioSourceHandle::new(" ");

    assert!(sound
        .submit_external_source_block(
            empty_handle,
            SoundExternalSourceBlock::new(48_000, SoundChannelLayout::mono(), vec![0.0]),
        )
        .unwrap_err()
        .to_string()
        .contains("external source handle"));
}
