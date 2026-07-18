use super::super::super::*;

fn assert_finite_error(result: Result<(), SoundError>) {
    let error = result.expect_err("non-finite automation values must be rejected");
    assert!(matches!(&error, SoundError::InvalidParameter(_)));
    assert!(error.to_string().contains("finite"), "{error}");
}

#[test]
fn direct_track_mute_automation_rejects_non_finite_input() {
    let sound = DefaultSoundManager::default();
    let binding = SoundAutomationBindingId::new(30);

    sound
        .bind_automation(SoundAutomationBinding {
            id: binding,
            timeline_track_path: "Root/Master:sound.master.mute".to_string(),
            target: SoundAutomationTarget::Track(SoundTrackId::master()),
            parameter: SoundParameterId::new("mute"),
        })
        .unwrap();

    assert_finite_error(sound.apply_automation_value(binding, f32::NAN));
}

#[test]
fn direct_volume_priority_automation_rejects_non_finite_input() {
    let sound = DefaultSoundManager::default();
    let volume = SoundVolumeId::new(31);
    let binding = SoundAutomationBindingId::new(31);

    sound
        .update_volume(SoundVolumeDescriptor {
            id: volume,
            shape: SoundVolumeShape::Sphere {
                center: [0.0; 3],
                radius: 1.0,
            },
            priority: 7,
            interior_gain: 1.0,
            exterior_gain: 1.0,
            low_pass_cutoff_hz: None,
            reverb_send: 0.0,
            convolution_send: None,
            crossfade_distance: 0.0,
        })
        .unwrap();
    sound
        .bind_automation(SoundAutomationBinding {
            id: binding,
            timeline_track_path: "Root/Volume:sound.volume.priority".to_string(),
            target: SoundAutomationTarget::Volume(volume),
            parameter: SoundParameterId::new("priority"),
        })
        .unwrap();

    assert_finite_error(sound.apply_automation_value(binding, f32::NAN));
}

#[test]
fn direct_chorus_voices_automation_rejects_non_finite_input_before_effect_lookup() {
    let sound = DefaultSoundManager::default();
    let binding = SoundAutomationBindingId::new(32);

    sound
        .bind_automation(SoundAutomationBinding {
            id: binding,
            timeline_track_path: "Root/Master:chorus.voices".to_string(),
            target: SoundAutomationTarget::Effect {
                track: SoundTrackId::master(),
                effect: SoundEffectId::new(32),
            },
            parameter: SoundParameterId::new("voices"),
        })
        .unwrap();

    assert_finite_error(sound.apply_automation_value(binding, f32::NAN));
}
