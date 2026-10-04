// 创建外部声源时拒绝空白句柄，核对声源描述符的输入身份约束。
use super::super::super::super::*;

#[test]
fn external_audio_source_descriptor_rejects_blank_handle() {
    let sound = DefaultSoundManager::default();
    let empty_handle = ExternalAudioSourceHandle::new(" ");

    assert!(sound
        .create_source(SoundSourceDescriptor {
            input: SoundSourceInput::External(empty_handle),
            ..SoundSourceDescriptor::clip(SoundClipId::new(999))
        })
        .unwrap_err()
        .to_string()
        .contains("external source handle"));
}
