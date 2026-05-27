use super::*;

#[test]
fn synth_parameter_source_and_timeline_binding_are_visible_in_snapshot() {
    let sound = DefaultSoundManager::default();
    let parameter = SoundParameterId::new("synth.cutoff");
    assert!(sound
        .parameter_value(&parameter)
        .unwrap_err()
        .to_string()
        .contains("unknown sound parameter"));
    sound.set_parameter(parameter.clone(), 0.25).unwrap();
    assert_sample_near(sound.parameter_value(&parameter).unwrap(), 0.25);
    let source = SoundSourceDescriptor {
        input: SoundSourceInput::SynthParameter {
            parameter: parameter.clone(),
            default_value: 0.0,
        },
        ..SoundSourceDescriptor::clip(SoundClipId::new(999))
    };
    let source_id = sound.create_source(source).unwrap();
    sound
        .bind_automation(SoundAutomationBinding {
            id: SoundAutomationBindingId::new(1),
            timeline_track_path: "Root/Synth:sound.synth.cutoff".to_string(),
            target: SoundAutomationTarget::SynthParameter(parameter),
            parameter: SoundParameterId::new("value"),
        })
        .unwrap();

    let mix = sound.render_mix(1).unwrap();
    let snapshot = sound.mixer_snapshot().unwrap();

    assert_eq!(mix.samples, vec![0.25, 0.25]);
    assert!(snapshot
        .graph
        .sources
        .iter()
        .any(|source| source.id == Some(source_id)));
    assert_eq!(snapshot.graph.automation_bindings.len(), 1);
    assert!(snapshot.graph.dynamic_events.events.is_empty());
}

#[test]
fn automation_binding_applies_values_to_synth_track_and_effect_targets() {
    let sound = DefaultSoundManager::default();
    let synth_parameter = SoundParameterId::new("synth.amp");
    let source = SoundSourceDescriptor {
        input: SoundSourceInput::SynthParameter {
            parameter: synth_parameter.clone(),
            default_value: 0.0,
        },
        ..SoundSourceDescriptor::clip(SoundClipId::new(999))
    };
    sound.create_source(source).unwrap();

    let synth_binding = SoundAutomationBindingId::new(10);
    sound
        .bind_automation(SoundAutomationBinding {
            id: synth_binding,
            timeline_track_path: "Root/Synth:sound.synth.amp".to_string(),
            target: SoundAutomationTarget::SynthParameter(synth_parameter.clone()),
            parameter: SoundParameterId::new("value"),
        })
        .unwrap();
    sound.apply_automation_value(synth_binding, 0.4).unwrap();
    assert_sample_near(sound.parameter_value(&synth_parameter).unwrap(), 0.4);
    assert_samples_near(&sound.render_mix(1).unwrap().samples, &[0.4, 0.4]);

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
    assert_samples_near(&sound.render_mix(1).unwrap().samples, &[0.2, 0.2]);

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

#[test]
fn automation_binding_reports_invalid_paths_and_targets_cleanly() {
    let sound = DefaultSoundManager::default();

    let missing = sound
        .apply_automation_value(SoundAutomationBindingId::new(999), 0.1)
        .unwrap_err();
    assert!(matches!(
        missing,
        SoundError::UnknownAutomationBinding { .. }
    ));
    assert!(sound
        .bind_automation(SoundAutomationBinding {
            id: SoundAutomationBindingId::new(20),
            timeline_track_path: " ".to_string(),
            target: SoundAutomationTarget::Track(SoundTrackId::master()),
            parameter: SoundParameterId::new("gain"),
        })
        .unwrap_err()
        .to_string()
        .contains("timeline track path"));

    let unsupported_binding = SoundAutomationBindingId::new(21);
    sound
        .bind_automation(SoundAutomationBinding {
            id: unsupported_binding,
            timeline_track_path: "Root/Master:sound.master.unknown".to_string(),
            target: SoundAutomationTarget::Track(SoundTrackId::master()),
            parameter: SoundParameterId::new("unknown_parameter"),
        })
        .unwrap();
    assert!(sound
        .apply_automation_value(unsupported_binding, 1.0)
        .unwrap_err()
        .to_string()
        .contains("unsupported sound automation parameter"));

    let unknown_source_binding = SoundAutomationBindingId::new(22);
    sound
        .bind_automation(SoundAutomationBinding {
            id: unknown_source_binding,
            timeline_track_path: "Root/Source:sound.source.gain".to_string(),
            target: SoundAutomationTarget::Source(SoundSourceId::new(404)),
            parameter: SoundParameterId::new("gain"),
        })
        .unwrap();
    assert!(matches!(
        sound
            .apply_automation_value(unknown_source_binding, 0.25)
            .unwrap_err(),
        SoundError::UnknownSource { .. }
    ));
}
