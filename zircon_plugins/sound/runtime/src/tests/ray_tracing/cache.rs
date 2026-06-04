use super::super::*;

#[test]
fn ray_traced_impulse_response_clear_invalidates_cache_and_static_ir() {
    let sound = DefaultSoundManager::default();
    let impulse_response = SoundImpulseResponseId::new(92);
    sound
        .submit_ray_traced_impulse_response(SoundRayTracedImpulseResponseDescriptor {
            impulse_response,
            cell_key: "cell-to-clear".to_string(),
            source: None,
            listener: None,
            volume: None,
            occlusion_gain: None,
            sample_rate_hz: 48_000,
            channel_count: 1,
            rays_traced: 64,
            samples: vec![1.0],
        })
        .unwrap();

    sound
        .clear_ray_traced_impulse_response(impulse_response)
        .unwrap();

    assert!(sound.ray_traced_impulse_responses().unwrap().is_empty());
    assert_eq!(
        sound.mixer_snapshot().unwrap().ray_tracing,
        SoundRayTracingConvolutionStatus::WaitingForGeometryProvider
    );
    assert!(matches!(
        sound.remove_impulse_response(impulse_response).unwrap_err(),
        SoundError::UnknownImpulseResponse { .. }
    ));
}
