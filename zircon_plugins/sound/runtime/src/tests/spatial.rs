use super::*;

fn test_hrtf_profile(profile_id: &str) -> SoundHrtfProfileDescriptor {
    SoundHrtfProfileDescriptor {
        profile_id: profile_id.to_string(),
        display_name: "Test HRTF".to_string(),
        sample_rate_hz: 48_000,
        left_kernel: vec![0.0, 0.5],
        right_kernel: vec![1.0],
        notes: vec!["unit-test profile".to_string()],
    }
}

fn long_tail_hrtf_profile(profile_id: &str) -> SoundHrtfProfileDescriptor {
    SoundHrtfProfileDescriptor {
        profile_id: profile_id.to_string(),
        display_name: "Long Tail Test HRTF".to_string(),
        sample_rate_hz: 48_000,
        left_kernel: vec![0.0, 0.0, 0.5],
        right_kernel: vec![1.0],
        notes: vec!["unit-test long-tail profile".to_string()],
    }
}

#[test]
fn hrtf_profiles_can_be_loaded_listed_and_removed() {
    let sound = DefaultSoundManager::default();
    sound
        .load_hrtf_profile(test_hrtf_profile("profile.b"))
        .unwrap();
    sound
        .load_hrtf_profile(test_hrtf_profile("profile.a"))
        .unwrap();

    let profiles = sound.hrtf_profiles().unwrap();
    assert_eq!(profiles.len(), 2);
    assert_eq!(profiles[0].profile_id, "profile.a");
    assert_eq!(profiles[1].profile_id, "profile.b");

    sound.remove_hrtf_profile("profile.a").unwrap();
    let profiles = sound.hrtf_profiles().unwrap();
    assert_eq!(profiles.len(), 1);
    assert_eq!(profiles[0].profile_id, "profile.b");
}

#[test]
fn hrtf_profile_validation_and_missing_remove_are_typed() {
    let sound = DefaultSoundManager::default();
    let mut invalid = test_hrtf_profile("bad");
    invalid.left_kernel = vec![f32::NAN];
    assert!(sound
        .load_hrtf_profile(invalid)
        .unwrap_err()
        .to_string()
        .contains("finite"));

    let mut silent = test_hrtf_profile("silent");
    silent.left_kernel = vec![0.0];
    silent.right_kernel = vec![0.0];
    assert!(sound
        .load_hrtf_profile(silent)
        .unwrap_err()
        .to_string()
        .contains("non-zero"));

    assert!(sound
        .remove_hrtf_profile("missing")
        .unwrap_err()
        .to_string()
        .contains("unknown HRTF profile"));
}

#[test]
fn loaded_hrtf_profile_applies_deterministic_kernels() {
    let sound = DefaultSoundManager::default();
    sound
        .load_hrtf_profile(test_hrtf_profile("loaded"))
        .unwrap();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/hrtf-loaded.wav", &[1.0, 0.0]));
    let mut listener = test_listener();
    listener.hrtf_profile = Some("loaded".to_string());
    sound.update_listener(listener).unwrap();

    let mut source = SoundSourceDescriptor::clip(clip);
    source.spatial = SoundSpatialSourceSettings {
        spatial_blend: 1.0,
        attenuation: SoundAttenuationMode::None,
        ..SoundSpatialSourceSettings::default()
    };
    sound.create_source(source).unwrap();

    let mix = sound.render_mix(2).unwrap();
    assert_sample_near(mix.samples[0], 0.0);
    assert_sample_near(mix.samples[1], 1.0);
    assert_sample_near(mix.samples[2], 0.5);
    assert_sample_near(mix.samples[3], 0.0);
}

#[test]
fn loaded_hrtf_profile_keeps_fir_tail_across_render_blocks() {
    let sound = DefaultSoundManager::default();
    sound
        .load_hrtf_profile(test_hrtf_profile("loaded-tail"))
        .unwrap();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/hrtf-tail.wav", &[1.0, 0.0]));
    let mut listener = test_listener();
    listener.hrtf_profile = Some("loaded-tail".to_string());
    sound.update_listener(listener).unwrap();

    let mut source = SoundSourceDescriptor::clip(clip);
    source.spatial = SoundSpatialSourceSettings {
        spatial_blend: 1.0,
        attenuation: SoundAttenuationMode::None,
        ..SoundSpatialSourceSettings::default()
    };
    sound.create_source(source).unwrap();

    assert_samples_near(&sound.render_mix(1).unwrap().samples, &[0.0, 1.0]);
    assert_samples_near(&sound.render_mix(1).unwrap().samples, &[0.5, 0.0]);
}

#[test]
fn loaded_hrtf_profile_renders_long_tail_after_source_completion() {
    let sound = DefaultSoundManager::default();
    sound
        .load_hrtf_profile(long_tail_hrtf_profile("loaded-long-tail"))
        .unwrap();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/hrtf-long-tail.wav", &[1.0]));
    let mut listener = test_listener();
    listener.hrtf_profile = Some("loaded-long-tail".to_string());
    sound.update_listener(listener).unwrap();

    let mut source = SoundSourceDescriptor::clip(clip);
    source.spatial = SoundSpatialSourceSettings {
        spatial_blend: 1.0,
        attenuation: SoundAttenuationMode::None,
        ..SoundSpatialSourceSettings::default()
    };
    let source_id = sound.create_source(source).unwrap();

    assert_samples_near(&sound.render_mix(1).unwrap().samples, &[0.0, 1.0]);
    assert_samples_near(&sound.render_mix(1).unwrap().samples, &[0.0, 0.0]);
    assert_samples_near(&sound.render_mix(1).unwrap().samples, &[0.5, 0.0]);
    assert!(sound.source_empty(source_id).unwrap());
}

#[test]
fn loaded_hrtf_profile_state_survives_parameter_driven_playing() {
    let sound = DefaultSoundManager::default();
    sound
        .load_hrtf_profile(test_hrtf_profile("parameter-loaded"))
        .unwrap();
    let playing_parameter = SoundParameterId::new("synth.hrtf_playing");
    sound.set_parameter(playing_parameter.clone(), 1.0).unwrap();
    let clip = sound.insert_clip_for_test(test_clip(
        "res://sound/hrtf-parameter-playing.wav",
        &[1.0, 0.0],
    ));
    let mut listener = test_listener();
    listener.hrtf_profile = Some("parameter-loaded".to_string());
    sound.update_listener(listener).unwrap();

    let mut source = SoundSourceDescriptor::clip(clip);
    source.playing = false;
    source.parameter_bindings.push(SoundSourceParameterBinding {
        source_parameter: SoundParameterId::new("playing"),
        synth_parameter: playing_parameter,
    });
    source.spatial = SoundSpatialSourceSettings {
        spatial_blend: 1.0,
        attenuation: SoundAttenuationMode::None,
        ..SoundSpatialSourceSettings::default()
    };
    sound.create_source(source).unwrap();

    assert_samples_near(&sound.render_mix(1).unwrap().samples, &[0.0, 1.0]);
    assert_samples_near(&sound.render_mix(1).unwrap().samples, &[0.5, 0.0]);
}

#[test]
fn missing_hrtf_profile_uses_preview_fallback_without_loaded_state() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/hrtf-missing.wav", &[1.0]));
    let mut listener = test_listener();
    listener.hrtf_profile = Some("missing-profile".to_string());
    listener.left_ear_offset = [-0.08, 0.0, 0.0];
    listener.right_ear_offset = [0.08, 0.0, 0.0];
    sound.update_listener(listener).unwrap();

    let mut source = SoundSourceDescriptor::clip(clip);
    source.position = [0.5, 0.0, 1.0];
    source.spatial = SoundSpatialSourceSettings {
        spatial_blend: 1.0,
        attenuation: SoundAttenuationMode::None,
        ..SoundSpatialSourceSettings::default()
    };
    sound.create_source(source).unwrap();

    let mix = sound.render_mix(16).unwrap();
    assert_sample_near(mix.samples[0], 0.0);
    assert!(mix.samples[1] > 0.9);
}

#[test]
fn hrtf_profile_applies_ear_delay_for_lateral_source() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/hrtf-preview.wav", &[1.0]));
    let mut listener = test_listener();
    listener.hrtf_profile = Some("preview".to_string());
    listener.left_ear_offset = [-0.08, 0.0, 0.0];
    listener.right_ear_offset = [0.08, 0.0, 0.0];
    sound.update_listener(listener).unwrap();

    let mut source = SoundSourceDescriptor::clip(clip);
    source.position = [0.5, 0.0, 1.0];
    source.spatial = SoundSpatialSourceSettings {
        spatial_blend: 1.0,
        attenuation: SoundAttenuationMode::None,
        ..SoundSpatialSourceSettings::default()
    };
    sound.create_source(source).unwrap();

    let mix = sound.render_mix(16).unwrap();
    assert_sample_near(mix.samples[0], 0.0);
    assert!(mix.samples[1] > 0.9);
    let delayed_left_peak = mix
        .samples
        .chunks_exact(2)
        .skip(1)
        .map(|frame| frame[0].abs())
        .fold(0.0_f32, f32::max);
    assert!(delayed_left_peak > 0.25);
}

#[test]
fn default_spatial_scale_controls_listener_source_distance() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/spatial-scale.wav", &[1.0]));
    sound.update_listener(test_listener()).unwrap();
    assert_eq!(sound.default_spatial_scale().unwrap(), 1.0);

    let mut source = SoundSourceDescriptor::clip(clip);
    source.position = [2.0, 0.0, 0.0];
    source.spatial = SoundSpatialSourceSettings {
        spatial_blend: 1.0,
        min_distance: 1.0,
        max_distance: 5.0,
        attenuation: SoundAttenuationMode::Linear,
        ..SoundSpatialSourceSettings::default()
    };
    sound.create_source(source).unwrap();

    let mix = sound.render_mix(1).unwrap();
    assert_sample_near(mix.samples[0], 0.0);
    assert_sample_near(mix.samples[1], 0.75);

    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/spatial-scale-half.wav", &[1.0]));
    sound.update_listener(test_listener()).unwrap();
    sound.set_default_spatial_scale(0.5).unwrap();
    assert_eq!(sound.default_spatial_scale().unwrap(), 0.5);
    assert!(sound.set_default_spatial_scale(f32::NAN).is_err());
    assert!(sound.set_default_spatial_scale(-0.1).is_err());

    let mut source = SoundSourceDescriptor::clip(clip);
    source.position = [2.0, 0.0, 0.0];
    source.spatial = SoundSpatialSourceSettings {
        spatial_blend: 1.0,
        min_distance: 1.0,
        max_distance: 5.0,
        attenuation: SoundAttenuationMode::Linear,
        ..SoundSpatialSourceSettings::default()
    };
    sound.create_source(source).unwrap();

    let mix = sound.render_mix(1).unwrap();
    assert_sample_near(mix.samples[0], 0.0);
    assert_sample_near(mix.samples[1], 1.0);
}

#[test]
fn source_spatial_scale_overrides_default_spatial_scale() {
    let sound = DefaultSoundManager::default();
    let clip =
        sound.insert_clip_for_test(test_clip("res://sound/source-spatial-scale.wav", &[1.0]));
    sound.update_listener(test_listener()).unwrap();
    sound.set_default_spatial_scale(0.5).unwrap();

    let mut source = SoundSourceDescriptor::clip(clip);
    source.position = [2.0, 0.0, 0.0];
    source.spatial = SoundSpatialSourceSettings {
        spatial_blend: 1.0,
        spatial_scale: Some(1.0),
        min_distance: 1.0,
        max_distance: 5.0,
        attenuation: SoundAttenuationMode::Linear,
        ..SoundSpatialSourceSettings::default()
    };
    sound.create_source(source).unwrap();

    let mix = sound.render_mix(1).unwrap();
    assert_sample_near(mix.samples[0], 0.0);
    assert_sample_near(mix.samples[1], 0.75);

    let mut invalid = SoundSourceDescriptor::clip(clip);
    invalid.spatial.spatial_scale = Some(f32::NAN);
    assert!(sound.create_source(invalid).is_err());

    let mut invalid = SoundSourceDescriptor::clip(clip);
    invalid.spatial.spatial_scale = Some(-0.1);
    assert!(sound.create_source(invalid).is_err());
}

#[test]
fn spatial_source_uses_active_listener_for_attenuation_pan_and_occlusion() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/spatial.wav", &[1.0]));
    sound.update_listener(test_listener()).unwrap();

    let mut source = SoundSourceDescriptor::clip(clip);
    source.position = [3.0, 0.0, 0.0];
    source.spatial = SoundSpatialSourceSettings {
        spatial_blend: 1.0,
        min_distance: 1.0,
        max_distance: 5.0,
        attenuation: SoundAttenuationMode::Linear,
        occlusion_enabled: true,
        ..SoundSpatialSourceSettings::default()
    };
    sound.create_source(source).unwrap();

    let mix = sound.render_mix(1).unwrap();

    assert_sample_near(mix.samples[0], 0.0);
    assert_sample_near(mix.samples[1], 0.35);
}

#[test]
fn audio_volume_priority_and_crossfade_apply_to_source_output() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/volume.wav", &[1.0]));
    let mut source = SoundSourceDescriptor::clip(clip);
    source.position = [2.0, 0.0, 0.0];
    sound.create_source(source).unwrap();
    sound
        .update_volume(SoundVolumeDescriptor {
            id: SoundVolumeId::new(1),
            shape: SoundVolumeShape::Sphere {
                center: [0.0, 0.0, 0.0],
                radius: 5.0,
            },
            priority: 0,
            interior_gain: 0.1,
            exterior_gain: 1.0,
            low_pass_cutoff_hz: None,
            reverb_send: 0.0,
            convolution_send: None,
            crossfade_distance: 0.0,
        })
        .unwrap();
    sound
        .update_volume(SoundVolumeDescriptor {
            id: SoundVolumeId::new(2),
            shape: SoundVolumeShape::Sphere {
                center: [0.0, 0.0, 0.0],
                radius: 1.0,
            },
            priority: 10,
            interior_gain: 0.25,
            exterior_gain: 1.0,
            low_pass_cutoff_hz: None,
            reverb_send: 0.0,
            convolution_send: None,
            crossfade_distance: 3.0,
        })
        .unwrap();

    let mix = sound.render_mix(1).unwrap();

    assert_sample_near(mix.samples[0], 0.5);
    assert_sample_near(mix.samples[1], 0.5);
}

#[test]
fn source_sends_can_tap_pre_spatial_signal() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/pre-spatial.wav", &[0.5]));
    let room = SoundTrackId::new(2);
    sound
        .add_or_update_track(SoundTrackDescriptor::child(room, "Room"))
        .unwrap();
    sound.update_listener(test_listener()).unwrap();

    let mut source = SoundSourceDescriptor::clip(clip);
    source.position = [3.0, 0.0, 0.0];
    source.sends.push(SoundSourceSend {
        target: room,
        gain: 1.0,
        pre_spatial: true,
    });
    source.spatial = SoundSpatialSourceSettings {
        spatial_blend: 1.0,
        min_distance: 1.0,
        max_distance: 5.0,
        attenuation: SoundAttenuationMode::Linear,
        ..SoundSpatialSourceSettings::default()
    };
    sound.create_source(source).unwrap();

    let mix = sound.render_mix(1).unwrap();

    assert_sample_near(mix.samples[0], 0.5);
    assert_sample_near(mix.samples[1], 0.75);
}
