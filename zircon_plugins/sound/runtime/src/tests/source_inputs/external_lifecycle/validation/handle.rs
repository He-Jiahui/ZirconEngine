// 经外部采样块提交入口拒绝空白句柄，核对外部输入身份的前置条件。
use super::super::super::super::*;

#[test]
fn external_audio_source_block_rejects_blank_handle() {
    let sound = DefaultSoundManager::default();
    let empty_handle = ExternalAudioSourceHandle::new(" ");

    assert!(sound
        .submit_external_source_block(
            empty_handle,
            SoundExternalSourceBlock::new(48_000, AudioChannelLayout::mono(), vec![0.0]),
        )
        .unwrap_err()
        .to_string()
        .contains("external source handle"));
}
