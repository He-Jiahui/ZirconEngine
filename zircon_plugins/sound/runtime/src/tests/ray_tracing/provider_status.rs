use super::super::*;

#[test]
fn ray_traced_impulse_response_submission_feeds_convolution_and_status() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/ray-ir.wav", &[0.5]));
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
        occlusion_gain: None,
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
    assert_samples_near(&sound.render_mix(1).unwrap().samples, &[0.75, 0.75]);
}
