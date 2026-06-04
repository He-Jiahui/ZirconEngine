use zircon_runtime::core::framework::sound::{
    SoundAutomationBinding, SoundAutomationBindingId, SoundAutomationTarget, SoundMixerGraph,
    SoundMixerGraphManager, SoundParameterId, SoundSourceDescriptor, SoundSourceId, SoundTrackId,
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
