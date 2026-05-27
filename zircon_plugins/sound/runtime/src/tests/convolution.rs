use super::*;

#[test]
fn static_convolution_impulse_response_processes_master_track() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/impulse.wav", &[1.0, 0.0]));
    let impulse_response = SoundImpulseResponseId::new(1);
    sound
        .set_impulse_response(impulse_response, vec![0.5, 0.25])
        .unwrap();
    sound
        .add_or_update_effect(
            SoundTrackId::master(),
            SoundEffectDescriptor::new(
                SoundEffectId::new(3),
                "Static IR",
                SoundEffectKind::ConvolutionReverb(SoundConvolutionReverbEffect {
                    impulse_response,
                    fallback_to_algorithmic: true,
                    latency_frames: 1,
                }),
            ),
        )
        .unwrap();
    sound
        .play_clip(clip, SoundPlaybackSettings::default())
        .unwrap();

    let mix = sound.render_mix(2).unwrap();

    assert_eq!(mix.samples, vec![0.5, 0.5, 0.25, 0.25]);
}

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

#[test]
fn ray_tracing_convolution_status_is_visible_and_validated() {
    let sound = DefaultSoundManager::default();

    sound
        .set_ray_tracing_convolution_status(
            SoundRayTracingConvolutionStatus::WaitingForGeometryProvider,
        )
        .unwrap();
    assert_eq!(
        sound.mixer_snapshot().unwrap().ray_tracing,
        SoundRayTracingConvolutionStatus::WaitingForGeometryProvider
    );

    sound
        .set_ray_tracing_convolution_status(SoundRayTracingConvolutionStatus::RayTraced {
            cached_cells: 2,
            rays_per_update: 64,
        })
        .unwrap();
    assert_eq!(
        sound.mixer_snapshot().unwrap().ray_tracing,
        SoundRayTracingConvolutionStatus::RayTraced {
            cached_cells: 2,
            rays_per_update: 64,
        }
    );
    assert!(sound
        .set_ray_tracing_convolution_status(SoundRayTracingConvolutionStatus::RayTraced {
            cached_cells: 2,
            rays_per_update: 0,
        })
        .unwrap_err()
        .to_string()
        .contains("ray"));
}
