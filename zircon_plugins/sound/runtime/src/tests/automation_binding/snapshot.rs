use super::super::*;

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
