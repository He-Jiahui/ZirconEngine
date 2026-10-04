// 经公共声管理器确认未知外部源清理报错、已提交块可清理；不覆盖无效描述符校验。
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
            SoundExternalSourceBlock::new(48_000, AudioChannelLayout::mono(), vec![0.75]),
        )
        .unwrap();
    sound.clear_external_source(&handle).unwrap();
}
