use super::*;

#[test]
fn animation_state_machine_event_marks_open_graph_editor_dirty_and_updates_transition_summary() {
    let _guard = env_lock().lock().unwrap();
    let mut harness = EventRuntimeHarness::new("zircon_editor_event_animation_state_machine_dirty");
    let asset_locator = open_indexed_animation_asset(
        &mut harness,
        "zircon_editor_event_animation_state_machine_asset_project",
        "res://animation/hero.state_machine.zranim",
        write_state_machine_asset,
    );

    harness
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Asset(EditorAssetEvent::OpenAsset {
                asset_locator: asset_locator.clone(),
            }),
        )
        .unwrap();
    harness
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Animation(
                crate::core::editor_event::EditorAnimationEvent::CreateTransition {
                    state_machine_locator: asset_locator.clone(),
                    from_state: "Idle".to_string(),
                    to_state: "Run".to_string(),
                    duration_frames: 8,
                },
            ),
        )
        .unwrap();

    let manager = harness
        .core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let instance = harness
        .runtime
        .current_view_instances()
        .into_iter()
        .find(|instance| instance.descriptor_id == ViewDescriptorId::new("editor.animation_graph"))
        .expect("graph editor view should stay open");
    let pane = manager
        .animation_editor_pane_presentation(&instance.instance_id)
        .expect("graph editor session should be queryable after command");

    assert!(
        instance.dirty,
        "state-machine authoring command should mark instance dirty"
    );
    assert_eq!(pane.transition_items, vec!["Idle -> Run"]);
    assert_eq!(
        harness.runtime.editor_snapshot().status_line,
        format!(
            "Created animation transition Idle -> Run in {} (8 frames)",
            asset_locator
        )
    );
}

#[test]
fn animation_state_machine_ignores_missing_entry_state_requests() {
    let _guard = env_lock().lock().unwrap();
    let mut harness =
        EventRuntimeHarness::new("zircon_editor_event_animation_state_machine_invalid_entry");
    let asset_locator = open_indexed_animation_asset(
        &mut harness,
        "zircon_editor_event_animation_state_machine_invalid_entry_project",
        "res://animation/hero.state_machine.zranim",
        write_state_machine_asset,
    );

    harness
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Asset(EditorAssetEvent::OpenAsset {
                asset_locator: asset_locator.clone(),
            }),
        )
        .unwrap();
    harness
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Animation(
                crate::core::editor_event::EditorAnimationEvent::SetEntryState {
                    state_machine_locator: asset_locator,
                    state_name: "Jump".to_string(),
                },
            ),
        )
        .unwrap();

    let manager = harness
        .core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let instance = harness
        .runtime
        .current_view_instances()
        .into_iter()
        .find(|instance| instance.descriptor_id == ViewDescriptorId::new("editor.animation_graph"))
        .expect("graph editor view should stay open");
    let pane = manager
        .animation_editor_pane_presentation(&instance.instance_id)
        .expect("graph editor session should remain queryable after invalid entry request");

    assert_eq!(
        pane.selection_summary, "Idle",
        "invalid entry-state request should preserve the current entry state"
    );
    assert!(
        !instance.dirty,
        "invalid entry-state request should remain a no-op"
    );
    assert_eq!(
        harness.runtime.editor_snapshot().status_line,
        "Ignored animation command because it did not change the current document"
    );
}

#[test]
fn animation_state_machine_ignores_transition_requests_with_missing_states() {
    let _guard = env_lock().lock().unwrap();
    let mut harness =
        EventRuntimeHarness::new("zircon_editor_event_animation_state_machine_invalid_transition");
    let asset_locator = open_indexed_animation_asset(
        &mut harness,
        "zircon_editor_event_animation_state_machine_invalid_transition_project",
        "res://animation/hero.state_machine.zranim",
        write_state_machine_asset,
    );

    harness
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Asset(EditorAssetEvent::OpenAsset {
                asset_locator: asset_locator.clone(),
            }),
        )
        .unwrap();
    harness
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Animation(
                crate::core::editor_event::EditorAnimationEvent::CreateTransition {
                    state_machine_locator: asset_locator,
                    from_state: "Idle".to_string(),
                    to_state: "Jump".to_string(),
                    duration_frames: 8,
                },
            ),
        )
        .unwrap();

    let manager = harness
        .core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let instance = harness
        .runtime
        .current_view_instances()
        .into_iter()
        .find(|instance| instance.descriptor_id == ViewDescriptorId::new("editor.animation_graph"))
        .expect("graph editor view should stay open");
    let pane = manager
        .animation_editor_pane_presentation(&instance.instance_id)
        .expect("graph editor session should remain queryable after invalid transition request");

    assert!(
        pane.transition_items.is_empty(),
        "invalid transition request should not create orphaned transitions"
    );
    assert!(
        !instance.dirty,
        "invalid transition request should remain a no-op"
    );
    assert_eq!(
        harness.runtime.editor_snapshot().status_line,
        "Ignored animation command because it did not change the current document"
    );
}

#[test]
fn animation_state_machine_ignores_condition_requests_for_missing_transitions() {
    let _guard = env_lock().lock().unwrap();
    let mut harness = EventRuntimeHarness::new(
        "zircon_editor_event_animation_state_machine_missing_transition_condition",
    );
    let asset_locator = open_indexed_animation_asset(
        &mut harness,
        "zircon_editor_event_animation_state_machine_missing_transition_project",
        "res://animation/hero.state_machine.zranim",
        write_state_machine_asset,
    );

    harness
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Asset(EditorAssetEvent::OpenAsset {
                asset_locator: asset_locator.clone(),
            }),
        )
        .unwrap();
    harness
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Animation(
                crate::core::editor_event::EditorAnimationEvent::SetTransitionCondition {
                    state_machine_locator: asset_locator,
                    from_state: "Idle".to_string(),
                    to_state: "Run".to_string(),
                    parameter_name: "speed".to_string(),
                    operator: "greater_equal".to_string(),
                    value_literal: "1.0".to_string(),
                },
            ),
        )
        .unwrap();

    let manager = harness
        .core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let instance = harness
        .runtime
        .current_view_instances()
        .into_iter()
        .find(|instance| instance.descriptor_id == ViewDescriptorId::new("editor.animation_graph"))
        .expect("graph editor view should stay open");
    let pane = manager
        .animation_editor_pane_presentation(&instance.instance_id)
        .expect("graph editor session should remain queryable after missing-transition request");

    assert!(
        pane.transition_items.is_empty(),
        "condition authoring should not create implicit transitions"
    );
    assert!(
        !instance.dirty,
        "missing-transition condition request should remain a no-op"
    );
    assert_eq!(
        harness.runtime.editor_snapshot().status_line,
        "Ignored animation command because it did not change the current document"
    );
}

#[test]
fn animation_state_machine_ignores_unknown_transition_condition_operator() {
    let _guard = env_lock().lock().unwrap();
    let mut harness = EventRuntimeHarness::new(
        "zircon_editor_event_animation_state_machine_unknown_condition_operator",
    );
    let asset_locator = open_indexed_animation_asset(
        &mut harness,
        "zircon_editor_event_animation_state_machine_unknown_condition_project",
        "res://animation/hero.state_machine.zranim",
        write_state_machine_asset_with_transition,
    );

    harness
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Asset(EditorAssetEvent::OpenAsset {
                asset_locator: asset_locator.clone(),
            }),
        )
        .unwrap();
    harness
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Animation(
                crate::core::editor_event::EditorAnimationEvent::SetTransitionCondition {
                    state_machine_locator: asset_locator,
                    from_state: "Idle".to_string(),
                    to_state: "Run".to_string(),
                    parameter_name: "speed".to_string(),
                    operator: "approximately".to_string(),
                    value_literal: "2.5".to_string(),
                },
            ),
        )
        .unwrap();

    let manager = harness
        .core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let instance = harness
        .runtime
        .current_view_instances()
        .into_iter()
        .find(|instance| instance.descriptor_id == ViewDescriptorId::new("editor.animation_graph"))
        .expect("graph editor view should stay open");
    let pane = manager
        .animation_editor_pane_presentation(&instance.instance_id)
        .expect("graph editor session should remain queryable after invalid condition operator");

    assert_eq!(
        pane.transition_items,
        vec!["Idle -> Run"],
        "unknown condition operator should preserve the existing transition summary"
    );
    assert!(
        !instance.dirty,
        "unknown condition operator should remain a no-op for the document"
    );
    assert_eq!(
        harness.runtime.editor_snapshot().status_line,
        "Ignored animation command because it did not change the current document"
    );
}
