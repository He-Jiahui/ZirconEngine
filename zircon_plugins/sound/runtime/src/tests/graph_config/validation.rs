use zircon_runtime::core::framework::sound::{
    SoundAutomationBinding, SoundAutomationBindingId, SoundAutomationTarget, SoundMixerGraph,
    SoundMixerGraphManager, SoundParameterId, SoundSourceDescriptor, SoundSourceId,
    SoundSourceInput, SoundSourceManager, SoundTrackId,
};

use super::super::{test_clip, DefaultSoundManager};

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

    let existing_source = sound
        .create_source(SoundSourceDescriptor::clip(clip))
        .unwrap();
    assert_eq!(existing_source, SoundSourceId::new(1));

    let mut invalid_binding_graph = SoundMixerGraph::default_stereo(48_000);
    invalid_binding_graph
        .automation_bindings
        .push(SoundAutomationBinding {
            id: SoundAutomationBindingId::new(12),
            timeline_track_path: " ".to_string(),
            target: SoundAutomationTarget::Track(SoundTrackId::master()),
            parameter: SoundParameterId::new("gain"),
        });
    invalid_binding_graph
        .sources
        .push(SoundSourceDescriptor::clip(clip));
    assert!(sound
        .configure_mixer(invalid_binding_graph)
        .unwrap_err()
        .to_string()
        .contains("timeline track path"));
    assert_eq!(
        sound.source_status(existing_source).unwrap().source,
        existing_source
    );

    let source = sound
        .create_source(SoundSourceDescriptor::clip(clip))
        .unwrap();
    assert_eq!(source, SoundSourceId::new(2));
}

#[test]
fn configure_mixer_atomically_replaces_sources_and_commits_the_id_cursor() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/configured.wav", &[0.5]));
    let old_source = sound
        .create_source(SoundSourceDescriptor::clip(clip))
        .unwrap();
    let configured_source = SoundSourceId::new(2);
    let mut graph = SoundMixerGraph::default_stereo(48_000);
    graph.sources.push(SoundSourceDescriptor::clip(clip));

    sound.configure_mixer(graph).unwrap();

    assert!(sound.source_status(old_source).is_err());
    let status = sound.source_status(configured_source).unwrap();
    assert_eq!(status.source, configured_source);
    assert_eq!(status.input, SoundSourceInput::Clip(clip));
    assert_eq!(
        sound.mixer_snapshot().unwrap().graph.sources[0].id,
        Some(configured_source)
    );
    let next_source = sound
        .create_source(SoundSourceDescriptor::clip(clip))
        .unwrap();
    assert_eq!(next_source, SoundSourceId::new(3));
}
