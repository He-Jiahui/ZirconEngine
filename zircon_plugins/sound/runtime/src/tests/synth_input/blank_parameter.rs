// 创建合成参数声源时拒绝空白参数身份，核对输入描述符的引用前置条件。
use super::super::super::super::*;

#[test]
fn synth_parameter_source_input_rejects_blank_parameter() {
    let sound = DefaultSoundManager::default();

    assert!(sound
        .create_source(SoundSourceDescriptor {
            input: SoundSourceInput::SynthParameter {
                parameter: SoundParameterId::new(" "),
                default_value: 0.0,
            },
            ..SoundSourceDescriptor::clip(SoundClipId::new(999))
        })
        .unwrap_err()
        .to_string()
        .contains("synth source input"));
}
