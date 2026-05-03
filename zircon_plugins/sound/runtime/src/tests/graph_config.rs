use zircon_runtime::core::framework::sound::{
    SoundAutomationBinding, SoundAutomationBindingId, SoundAutomationTarget, SoundManager,
    SoundMixerGraph, SoundParameterId, SoundSourceDescriptor, SoundSourceId, SoundTrackDescriptor,
    SoundTrackId,
};

use super::{assert_samples_near, test_clip, DefaultSoundManager};

#[test]
fn configure_mixer_imports_graph_sources_and_automation_bindings() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/configured.wav", &[1.0]));
    let music = SoundTrackId::new(42);
    let source_id = SoundSourceId::new(77);
    let binding_id = SoundAutomationBindingId::new(9);

    let mut graph = SoundMixerGraph::default_stereo(48_000);
    graph
        .tracks
        .push(SoundTrackDescriptor::child(music, "Configured Music"));
    let mut source = SoundSourceDescriptor::clip(clip);
    source.id = Some(source_id);
    source.output_track = music;
    source.gain = 0.25;
    source.looped = true;
    graph.sources.push(source);
    graph.automation_bindings.push(SoundAutomationBinding {
        id: binding_id,
        timeline_track_path: "Root/Sound/ConfiguredSource:gain".to_string(),
        target: SoundAutomationTarget::Source(source_id),
        parameter: SoundParameterId::new("gain"),
    });

    sound.configure_mixer(graph).unwrap();

    let snapshot = sound.mixer_snapshot().unwrap();
    assert_eq!(snapshot.graph.sources.len(), 1);
    assert_eq!(snapshot.graph.sources[0].id, Some(source_id));
    assert_eq!(snapshot.graph.automation_bindings.len(), 1);
    assert_samples_near(&sound.render_mix(1).unwrap().samples, &[0.25, 0.25]);

    sound.apply_automation_value(binding_id, 0.5).unwrap();
    assert_samples_near(&sound.render_mix(1).unwrap().samples, &[0.5, 0.5]);
}

#[test]
fn configure_mixer_rejects_duplicate_sources_and_invalid_bindings() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/duplicate.wav", &[1.0]));

    let mut duplicate_source_graph = SoundMixerGraph::default_stereo(48_000);
    let mut first_source = SoundSourceDescriptor::clip(clip);
    first_source.id = Some(SoundSourceId::new(3));
    let mut second_source = SoundSourceDescriptor::clip(clip);
    second_source.id = Some(SoundSourceId::new(3));
    duplicate_source_graph.sources.push(first_source);
    duplicate_source_graph.sources.push(second_source);

    assert!(sound
        .configure_mixer(duplicate_source_graph)
        .unwrap_err()
        .to_string()
        .contains("duplicate source ids"));

    let mut invalid_binding_graph = SoundMixerGraph::default_stereo(48_000);
    invalid_binding_graph
        .automation_bindings
        .push(SoundAutomationBinding {
            id: SoundAutomationBindingId::new(12),
            timeline_track_path: " ".to_string(),
            target: SoundAutomationTarget::Track(SoundTrackId::master()),
            parameter: SoundParameterId::new("gain"),
        });
    assert!(sound
        .configure_mixer(invalid_binding_graph)
        .unwrap_err()
        .to_string()
        .contains("timeline track path"));
}
