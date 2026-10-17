// 经停机管理器核对整图配置的声源与绑定校验，以及成功提交后的声源替换和 ID 游标。
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

#[test]
fn create_source_rejects_duplicate_ids_and_advances_cursor_after_success() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/source-ids.wav", &[0.5]));
    let replacement_clip =
        sound.insert_clip_for_test(test_clip("res://sound/replacement.wav", &[0.25]));
    let source_id = SoundSourceId::new(12);
    let mut explicit = SoundSourceDescriptor::clip(clip);
    explicit.id = Some(source_id);

    sound.create_source(explicit.clone()).unwrap();
    explicit.input = SoundSourceInput::Clip(replacement_clip);
    let error = sound.create_source(explicit).unwrap_err();
    assert!(error
        .to_string()
        .contains("sound source id 12 is already in use"));
    let original_status = sound.source_status(source_id).unwrap();
    assert_eq!(original_status.source, source_id);
    assert_eq!(original_status.input, SoundSourceInput::Clip(clip));

    let next_source = sound
        .create_source(SoundSourceDescriptor::clip(clip))
        .unwrap();
    assert_eq!(next_source, SoundSourceId::new(13));
}

#[test]
fn configure_mixer_reserves_later_explicit_ids_before_assigning_automatic_ids() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/reserved-source-ids.wav", &[0.5]));
    let mut graph = SoundMixerGraph::default_stereo(48_000);
    let mut automatic = SoundSourceDescriptor::clip(clip);
    automatic.gain = 0.5;
    graph.sources.push(automatic);
    let mut explicit = SoundSourceDescriptor::clip(clip);
    explicit.gain = 0.75;
    explicit.id = Some(SoundSourceId::new(1));
    graph.sources.push(explicit);

    sound.configure_mixer(graph).unwrap();

    let configured = sound.mixer_snapshot().unwrap().graph.sources;
    let automatic = configured
        .iter()
        .find(|source| source.gain == 0.5)
        .expect("automatic descriptor is identifiable independent of snapshot order");
    let explicit = configured
        .iter()
        .find(|source| source.gain == 0.75)
        .expect("explicit descriptor is identifiable independent of snapshot order");
    assert_eq!(automatic.id, Some(SoundSourceId::new(2)));
    assert_eq!(explicit.id, Some(SoundSourceId::new(1)));
    assert_ne!(automatic.id, explicit.id);

    let next_source = sound
        .create_source(SoundSourceDescriptor::clip(clip))
        .unwrap();
    assert_eq!(next_source, SoundSourceId::new(3));
}

#[test]
fn source_id_exhaustion_returns_an_error_without_wrapping() {
    let sound = DefaultSoundManager::default();
    let clip =
        sound.insert_clip_for_test(test_clip("res://sound/exhausted-source-ids.wav", &[0.5]));
    let mut max_id_source = SoundSourceDescriptor::clip(clip);
    max_id_source.id = Some(SoundSourceId::new(u64::MAX));
    sound.create_source(max_id_source).unwrap();

    let error = sound
        .create_source(SoundSourceDescriptor::clip(clip))
        .unwrap_err();
    assert!(error
        .to_string()
        .contains("sound source id space is exhausted"));
}

#[test]
fn configure_mixer_source_id_exhaustion_returns_an_error_without_wrapping() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip(
        "res://sound/exhausted-mixer-source-ids.wav",
        &[0.5],
    ));
    let mut max_id_source = SoundSourceDescriptor::clip(clip);
    max_id_source.id = Some(SoundSourceId::new(u64::MAX));
    sound.create_source(max_id_source).unwrap();

    let mut graph = SoundMixerGraph::default_stereo(48_000);
    graph.sources.push(SoundSourceDescriptor::clip(clip));

    let error = sound.configure_mixer(graph).unwrap_err();
    assert!(error
        .to_string()
        .contains("sound source id space is exhausted"));
}
