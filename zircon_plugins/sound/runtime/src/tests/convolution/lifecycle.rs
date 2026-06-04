use super::super::*;

#[test]
fn impulse_response_lifecycle_can_invalidate_static_convolution_cache() {
    let sound = DefaultSoundManager::default();
    let parameter = SoundParameterId::new("ir.input");
    let impulse_response = SoundImpulseResponseId::new(44);
    sound.set_parameter(parameter.clone(), 1.0).unwrap();
    sound
        .set_impulse_response(impulse_response, vec![0.5])
        .unwrap();
    sound
        .add_or_update_effect(
            SoundTrackId::master(),
            SoundEffectDescriptor::new(
                SoundEffectId::new(44),
                "Invalidate IR",
                SoundEffectKind::ConvolutionReverb(SoundConvolutionReverbEffect {
                    impulse_response,
                    fallback_to_algorithmic: false,
                    latency_frames: 0,
                }),
            ),
        )
        .unwrap();
    sound
        .create_source(SoundSourceDescriptor {
            input: SoundSourceInput::SynthParameter {
                parameter,
                default_value: 0.0,
            },
            ..SoundSourceDescriptor::clip(SoundClipId::new(999))
        })
        .unwrap();

    assert_samples_near(&sound.render_mix(1).unwrap().samples, &[0.5, 0.5]);
    sound.remove_impulse_response(impulse_response).unwrap();
    assert_samples_near(&sound.render_mix(1).unwrap().samples, &[1.0, 1.0]);
    assert!(matches!(
        sound.remove_impulse_response(impulse_response).unwrap_err(),
        SoundError::UnknownImpulseResponse { .. }
    ));
}
