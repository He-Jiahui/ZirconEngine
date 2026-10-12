use std::sync::Arc;

use kira::{backend::mock::MockBackend, AudioManagerSettings};
use zircon_runtime::core::framework::sound::{SoundError, SoundTrackDescriptor, SoundTrackId};

use super::{
    lock_recover, mutate_graph, ActiveGraphCommitHarness, DefaultSoundManager, KiraEngine,
    SoundMixerGraph,
};

#[test]
fn retirement_after_graph_snapshot_rejects_graph_and_consumer_metadata_commit() {
    let manager = DefaultSoundManager::default();
    let (original_graph, original_revision, original_source_id) = {
        let state = lock_recover(&manager.state);
        (
            Arc::clone(&state.graph),
            state.graph_revision,
            state.next_source_id,
        )
    };
    let result = mutate_graph(
        &manager,
        |graph| {
            // mutate runs after the snapshot while the service lock is released.
            // Mark the provider pending before the actual final commit reacquires it.
            lock_recover(&manager.state)
                .kira
                .mark_provider_retiring_for_test();
            graph
                .tracks
                .push(SoundTrackDescriptor::child(SoundTrackId::new(7), "Pending"));
            Ok(())
        },
        |state| state.next_source_id = original_source_id + 1,
    );
    assert!(matches!(result, Err(SoundError::BackendUnavailable { .. })));
    let state = lock_recover(&manager.state);
    assert!(Arc::ptr_eq(&state.graph, &original_graph));
    assert_eq!(state.graph_revision, original_revision);
    assert_eq!(state.next_source_id, original_source_id);
}

#[test]
fn inactive_graph_retry_rejects_retiring_provider_and_retains_installed_graph() {
    let graph = SoundMixerGraph::default_stereo(48_000);
    let mut engine = KiraEngine::<MockBackend>::inactive();
    engine
        .activate(AudioManagerSettings::<MockBackend>::default())
        .unwrap();
    engine.sync_graph(&graph).unwrap();
    let harness = ActiveGraphCommitHarness::new(engine, graph);
    let original = Arc::clone(&lock_recover(&harness.state).graph);
    harness.with_kira_mut(|engine| engine.mark_provider_retiring_for_test());
    let mut replacement = SoundMixerGraph::default_stereo(48_000);
    replacement
        .tracks
        .push(SoundTrackDescriptor::child(SoundTrackId::new(8), "Retry"));
    // This is the second-snapshot state of the reported race: inactive, plan=None,
    // retained manager. It must reject before replace_graph or revision publication.
    assert!(matches!(
        harness.replace_graph(replacement),
        Err(SoundError::BackendUnavailable { .. })
    ));
    let state = lock_recover(&harness.state);
    assert_eq!(state.revision, 0);
    assert!(Arc::ptr_eq(&state.graph, &original));
    assert!(state.kira.installed_graph_for_test().is_some());
}

#[test]
fn preactivation_graph_edit_remains_available_before_provider_retirement() {
    let manager = DefaultSoundManager::default();
    let original_revision = lock_recover(&manager.state).graph_revision;
    mutate_graph(
        &manager,
        |graph| {
            graph.tracks.push(SoundTrackDescriptor::child(
                SoundTrackId::new(9),
                "Preactivation",
            ));
            Ok(())
        },
        |state| state.next_source_id = 19,
    )
    .unwrap();
    let state = lock_recover(&manager.state);
    assert!(!state.kira.is_active());
    assert!(state
        .graph
        .tracks
        .iter()
        .any(|track| track.id == SoundTrackId::new(9)));
    assert_ne!(state.graph_revision, original_revision);
    assert_eq!(state.next_source_id, 19);
}

#[test]
fn retiring_provider_rejects_automation_value_and_curve_without_graph_publication() {
    use zircon_runtime::core::framework::sound::{
        SoundAutomationBinding, SoundAutomationBindingId, SoundAutomationCurve,
        SoundAutomationKeyframe, SoundAutomationTarget, SoundParameterId,
    };

    let manager = DefaultSoundManager::default();
    let binding = SoundAutomationBindingId::new(21);
    manager
        .bind_automation_impl(SoundAutomationBinding {
            id: binding,
            timeline_track_path: "Timeline/retirement:sound.track.gain".to_string(),
            target: SoundAutomationTarget::Track(SoundTrackId::master()),
            parameter: SoundParameterId::new("gain"),
        })
        .unwrap();
    // Valid preactivation automation remains available before retirement.
    manager.apply_automation_value_impl(binding, 0.5).unwrap();
    let (original_graph, original_revision) = {
        let mut state = lock_recover(&manager.state);
        state.kira.mark_provider_retiring_for_test();
        (Arc::clone(&state.graph), state.graph_revision)
    };
    assert!(matches!(
        manager.apply_automation_value_impl(binding, 0.75),
        Err(SoundError::BackendUnavailable { .. })
    ));
    let curve = SoundAutomationCurve::from_keyframes([
        SoundAutomationKeyframe::linear(0.0, 0.25),
        SoundAutomationKeyframe::linear(1.0, 0.75),
    ]);
    assert!(matches!(
        manager.apply_automation_curve_sample_impl(binding, &curve, 0.5),
        Err(SoundError::BackendUnavailable { .. })
    ));
    let state = lock_recover(&manager.state);
    assert!(Arc::ptr_eq(&state.graph, &original_graph));
    assert_eq!(state.graph_revision, original_revision);
    assert!(state.automation_bindings.contains_key(&binding));
}
