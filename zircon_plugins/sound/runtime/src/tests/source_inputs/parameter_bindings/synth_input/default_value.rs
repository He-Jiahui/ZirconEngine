use super::super::super::super::*;

#[test]
fn synth_parameter_source_input_rejects_non_finite_default() {
    let sound = DefaultSoundManager::default();

    assert!(sound
        .create_source(SoundSourceDescriptor {
            input: SoundSourceInput::SynthParameter {
                parameter: SoundParameterId::new("synth.bad_default"),
                default_value: f32::NAN,
            },
            ..SoundSourceDescriptor::clip(SoundClipId::new(999))
        })
        .unwrap_err()
        .to_string()
        .contains("finite default"));
}
