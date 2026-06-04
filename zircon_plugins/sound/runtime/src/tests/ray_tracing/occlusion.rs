use super::super::*;

#[test]
fn ray_traced_impulse_response_occlusion_gain_replaces_static_fallback() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/ray-occlusion.wav", &[1.0]));
    sound.update_listener(test_listener()).unwrap();

    let mut source = SoundSourceDescriptor::clip(clip);
    source.position = [1.0, 0.0, 0.0];
    source.spatial = SoundSpatialSourceSettings {
        spatial_blend: 1.0,
        attenuation: SoundAttenuationMode::None,
        occlusion_enabled: true,
        ..SoundSpatialSourceSettings::default()
    };
    let source_id = sound.create_source(source).unwrap();

    sound
        .submit_ray_traced_impulse_response(SoundRayTracedImpulseResponseDescriptor {
            impulse_response: SoundImpulseResponseId::new(96),
            cell_key: "listener-1/source-1/occlusion".to_string(),
            source: Some(source_id),
            listener: Some(SoundListenerId::new(1)),
            volume: None,
            occlusion_gain: Some(0.25),
            sample_rate_hz: 48_000,
            channel_count: 1,
            rays_traced: 96,
            samples: vec![1.0],
        })
        .unwrap();

    assert_samples_near(&sound.render_mix(1).unwrap().samples, &[0.0, 0.25]);
}
