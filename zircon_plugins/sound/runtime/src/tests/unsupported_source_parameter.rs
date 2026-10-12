// 创建声源时拒绝不支持的绑定参数名称，核对声源参数绑定的允许集合。
use super::super::super::*;

#[test]
fn audio_source_parameter_binding_rejects_unsupported_source_parameter() {
    let sound = DefaultSoundManager::default();
    let mut invalid_source = SoundSourceDescriptor {
        input: SoundSourceInput::Silence,
        ..SoundSourceDescriptor::clip(SoundClipId::new(999))
    };
    invalid_source
        .parameter_bindings
        .push(SoundSourceParameterBinding {
            source_parameter: SoundParameterId::new("not_a_source_parameter"),
            synth_parameter: SoundParameterId::new("synth.invalid"),
        });

    assert!(sound
        .create_source(invalid_source)
        .unwrap_err()
        .to_string()
        .contains("unsupported source parameter binding"));
}
