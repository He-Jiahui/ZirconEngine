use super::super::super::*;

#[test]
fn automation_binding_applies_values_to_effect_targets() {
    let sound = DefaultSoundManager::default();
    let synth_parameter = SoundParameterId::new("synth.amp");
    let source = SoundSourceDescriptor {
        input: SoundSourceInput::SynthParameter {
            parameter: synth_parameter,
            default_value: 0.4,
        },
        ..SoundSourceDescriptor::clip(SoundClipId::new(999))
    };
    sound.create_source(source).unwrap();
    let track_binding = SoundAutomationBindingId::new(11);
    sound
        .bind_automation(SoundAutomationBinding {
            id: track_binding,
            timeline_track_path: "Root/Master:sound.master.gain".to_string(),
            target: SoundAutomationTarget::Track(SoundTrackId::master()),
            parameter: SoundParameterId::new("gain"),
        })
        .unwrap();
    sound.apply_automation_value(track_binding, 0.5).unwrap();

    let effect_id = SoundEffectId::new(88);
    sound
        .add_or_update_effect(
            SoundTrackId::master(),
            SoundEffectDescriptor::new(
                effect_id,
                "Automated Gain",
                SoundEffectKind::Gain(SoundGainEffect { gain: 0.0 }),
            ),
        )
        .unwrap();
    let effect_binding = SoundAutomationBindingId::new(12);
    sound
        .bind_automation(SoundAutomationBinding {
            id: effect_binding,
            timeline_track_path: "Root/Master/AutomatedGain:sound.effect.wet".to_string(),
            target: SoundAutomationTarget::Effect {
                track: SoundTrackId::master(),
                effect: effect_id,
            },
            parameter: SoundParameterId::new("wet"),
        })
        .unwrap();
    sound.apply_automation_value(effect_binding, 0.5).unwrap();

    assert_samples_near(&sound.render_mix(1).unwrap().samples, &[0.1, 0.1]);
}
