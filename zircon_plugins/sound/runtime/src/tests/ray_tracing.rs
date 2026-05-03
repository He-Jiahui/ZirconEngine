use super::*;

#[test]
fn ray_traced_impulse_response_submission_feeds_convolution_and_status() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/ray-ir.wav", &[1.0]));
    let mut source = SoundSourceDescriptor::clip(clip);
    source.spatial.spatial_blend = 1.0;
    source.spatial.convolution_send = Some(SoundImpulseResponseId::new(91));
    let source_id = sound.create_source(source).unwrap();
    sound.update_listener(test_listener()).unwrap();
    sound
        .update_volume(SoundVolumeDescriptor {
            id: SoundVolumeId::new(5),
            shape: SoundVolumeShape::Sphere {
                center: [0.0, 0.0, 0.0],
                radius: 2.0,
            },
            priority: 1,
            interior_gain: 1.0,
            exterior_gain: 1.0,
            low_pass_cutoff_hz: None,
            reverb_send: 0.0,
            convolution_send: None,
            crossfade_distance: 0.0,
        })
        .unwrap();

    let descriptor = SoundRayTracedImpulseResponseDescriptor {
        impulse_response: SoundImpulseResponseId::new(91),
        cell_key: "listener-1/source-1/room".to_string(),
        source: Some(source_id),
        listener: Some(SoundListenerId::new(1)),
        volume: Some(SoundVolumeId::new(5)),
        sample_rate_hz: 48_000,
        channel_count: 1,
        rays_traced: 128,
        samples: vec![0.5],
    };
    sound
        .submit_ray_traced_impulse_response(descriptor.clone())
        .unwrap();

    assert_eq!(
        sound.mixer_snapshot().unwrap().ray_tracing,
        SoundRayTracingConvolutionStatus::RayTraced {
            cached_cells: 1,
            rays_per_update: 128,
        }
    );
    assert_eq!(
        sound.ray_traced_impulse_responses().unwrap(),
        vec![descriptor]
    );
    assert_samples_near(&sound.render_mix(1).unwrap().samples, &[1.5, 1.5]);
}

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

#[test]
fn ray_traced_impulse_response_submission_validates_provider_data() {
    let sound = DefaultSoundManager::default();

    assert!(sound
        .submit_ray_traced_impulse_response(SoundRayTracedImpulseResponseDescriptor {
            impulse_response: SoundImpulseResponseId::new(93),
            cell_key: String::new(),
            source: None,
            listener: None,
            volume: None,
            sample_rate_hz: 48_000,
            channel_count: 1,
            rays_traced: 1,
            samples: vec![1.0],
        })
        .unwrap_err()
        .to_string()
        .contains("ray-traced impulse response"));
    assert!(matches!(
        sound
            .submit_ray_traced_impulse_response(SoundRayTracedImpulseResponseDescriptor {
                impulse_response: SoundImpulseResponseId::new(94),
                cell_key: "missing-source".to_string(),
                source: Some(SoundSourceId::new(404)),
                listener: None,
                volume: None,
                sample_rate_hz: 48_000,
                channel_count: 1,
                rays_traced: 1,
                samples: vec![1.0],
            })
            .unwrap_err(),
        SoundError::UnknownSource { .. }
    ));
    assert!(sound
        .submit_ray_traced_impulse_response(SoundRayTracedImpulseResponseDescriptor {
            impulse_response: SoundImpulseResponseId::new(95),
            cell_key: "bad-rays".to_string(),
            source: None,
            listener: None,
            volume: None,
            sample_rate_hz: 48_000,
            channel_count: 1,
            rays_traced: 0,
            samples: vec![1.0],
        })
        .unwrap_err()
        .to_string()
        .contains("ray-traced impulse response"));
}
