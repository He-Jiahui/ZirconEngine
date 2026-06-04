use super::super::*;

#[test]
fn automation_binding_normalizes_animation_track_paths_in_snapshot() {
    let sound = DefaultSoundManager::default();
    sound
        .bind_automation(SoundAutomationBinding {
            id: SoundAutomationBindingId::new(2),
            timeline_track_path: " Root / Synth : sound.synth.cutoff ".to_string(),
            target: SoundAutomationTarget::SynthParameter(SoundParameterId::new("synth.cutoff")),
            parameter: SoundParameterId::new("value"),
        })
        .unwrap();

    let snapshot = sound.mixer_snapshot().unwrap();
    assert_eq!(snapshot.graph.automation_bindings.len(), 1);
    assert_eq!(
        snapshot.graph.automation_bindings[0].timeline_track_path,
        "Root/Synth:sound.synth.cutoff"
    );
}
