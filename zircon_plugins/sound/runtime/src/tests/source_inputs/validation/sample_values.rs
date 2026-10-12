// 经外部采样块提交入口拒绝非有限样本，核对输入样本的数值边界。
use super::super::super::super::*;

#[test]
fn external_audio_source_block_rejects_non_finite_samples() {
    let sound = DefaultSoundManager::default();
    let handle = ExternalAudioSourceHandle::new("navigation.surface-noise");

    assert!(sound
        .submit_external_source_block(
            handle,
            SoundExternalSourceBlock::new(48_000, AudioChannelLayout::mono(), vec![f32::NAN]),
        )
        .unwrap_err()
        .to_string()
        .contains("finite"));
}
