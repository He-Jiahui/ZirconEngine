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
