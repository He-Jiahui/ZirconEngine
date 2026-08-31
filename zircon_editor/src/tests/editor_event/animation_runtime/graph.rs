use super::*;
use crate::core::editor_event::MenuAction;

#[test]
fn menu_undo_replays_the_focused_animation_document_history() {
    let _guard = env_lock().lock().unwrap();
    let mut harness = EventRuntimeHarness::new("zircon_editor_event_animation_graph_menu_undo");
    let asset_locator = open_indexed_animation_asset(
        &mut harness,
        "zircon_editor_event_animation_graph_menu_undo_project",
        "res://animation/hero.graph.zranim",
        write_graph_asset,
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
    let animation_record = harness
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Animation(
                crate::core::editor_event::EditorAnimationEvent::RemoveGraphNode {
                    graph_locator: asset_locator,
                    node_id: "output".to_string(),
                },
            ),
        )
        .unwrap();
    assert!(
        animation_record.transaction_id.is_some(),
        "a persistent animation mutation must record its document transaction identity"
    );
    assert!(
        harness.runtime.chrome_snapshot().can_undo,
        "the focused document history must enable the workbench undo command"
    );
    harness
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::WorkbenchMenu(MenuAction::Undo),
        )
        .expect("undo should replay the focused animation document history");

    let manager = harness
        .core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let instance = harness
        .runtime
        .current_view_instances()
        .into_iter()
        .find(|instance| instance.descriptor_id == ViewDescriptorId::new("editor.animation_graph"))
        .expect("graph editor view should remain open");
    let pane = manager
        .animation_editor_pane_presentation(&instance.instance_id)
        .expect("graph editor session should project the restored source");

    assert!(
        pane.node_items
            .iter()
            .any(|item| item.starts_with("Output <-")),
        "menu undo must restore the graph output node from document history"
    );
    assert!(
        !instance.dirty,
        "undoing back to the saved history mark must clear document dirty metadata"
    );
    assert_eq!(harness.runtime.editor_snapshot().status_line, "Undo");
}

#[test]
fn animation_graph_ignores_duplicate_output_node_requests() {
    let _guard = env_lock().lock().unwrap();
    let mut harness =
        EventRuntimeHarness::new("zircon_editor_event_animation_graph_duplicate_output");
    let asset_locator = open_indexed_animation_asset(
        &mut harness,
        "zircon_editor_event_animation_graph_duplicate_output_project",
        "res://animation/hero.graph.zranim",
        write_graph_asset,
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
                crate::core::editor_event::EditorAnimationEvent::AddGraphNode {
                    graph_locator: asset_locator.clone(),
                    node_id: "output_2".to_string(),
                    node_kind: "output".to_string(),
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
        .expect("graph editor session should remain queryable after duplicate output request");

    assert_eq!(
        pane.node_items
            .iter()
            .filter(|item| item.starts_with("Output <-"))
            .count(),
        1,
        "graph editor should preserve a single output node"
    );
    assert!(
        !instance.dirty,
        "duplicate output node request should remain a no-op"
    );
    assert_eq!(
        harness.runtime.editor_snapshot().status_line,
        "Ignored animation command because it did not change the current document"
    );
}

#[test]
fn animation_graph_removes_output_node_when_requested() {
    let _guard = env_lock().lock().unwrap();
    let mut harness = EventRuntimeHarness::new("zircon_editor_event_animation_graph_remove_output");
    let asset_locator = open_indexed_animation_asset(
        &mut harness,
        "zircon_editor_event_animation_graph_remove_output_project",
        "res://animation/hero.graph.zranim",
        write_graph_asset,
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
                crate::core::editor_event::EditorAnimationEvent::RemoveGraphNode {
                    graph_locator: asset_locator.clone(),
                    node_id: "output".to_string(),
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
        .expect("graph editor session should remain queryable after removing output");

    assert!(
        pane.node_items
            .iter()
            .all(|item| !item.starts_with("Output <-")),
        "removing the output node should clear it from the graph session"
    );
    assert!(
        instance.dirty,
        "removing the output node should mark the document dirty"
    );
    assert_eq!(
        harness.runtime.editor_snapshot().status_line,
        format!("Removed animation graph node output from {}", asset_locator)
    );
}

#[test]
fn animation_graph_ignores_connections_from_missing_source_nodes() {
    let _guard = env_lock().lock().unwrap();
    let mut harness =
        EventRuntimeHarness::new("zircon_editor_event_animation_graph_missing_source");
    let asset_locator = open_indexed_animation_asset(
        &mut harness,
        "zircon_editor_event_animation_graph_missing_source_project",
        "res://animation/hero.graph.zranim",
        write_graph_asset,
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
                crate::core::editor_event::EditorAnimationEvent::ConnectGraphNodes {
                    graph_locator: asset_locator.clone(),
                    from_node_id: "ghost".to_string(),
                    to_node_id: "locomotion".to_string(),
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
        .expect("graph editor session should remain queryable after invalid connection request");

    assert!(
        pane.node_items
            .iter()
            .any(|item| item == "Blend locomotion"),
        "invalid source connection should preserve the blend node's original inputs"
    );
    assert!(
        pane.node_items.iter().all(|item| !item.contains("ghost")),
        "invalid source connection should not write dangling node references"
    );
    assert!(
        !instance.dirty,
        "invalid source connection should remain a no-op"
    );
    assert_eq!(
        harness.runtime.editor_snapshot().status_line,
        "Ignored animation command because it did not change the current document"
    );
}

#[test]
fn animation_graph_ignores_self_referential_connections() {
    let _guard = env_lock().lock().unwrap();
    let mut harness = EventRuntimeHarness::new("zircon_editor_event_animation_graph_self_cycle");
    let asset_locator = open_indexed_animation_asset(
        &mut harness,
        "zircon_editor_event_animation_graph_self_cycle_project",
        "res://animation/hero.graph.zranim",
        write_graph_asset,
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
                crate::core::editor_event::EditorAnimationEvent::ConnectGraphNodes {
                    graph_locator: asset_locator.clone(),
                    from_node_id: "locomotion".to_string(),
                    to_node_id: "locomotion".to_string(),
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
        .expect("graph editor session should remain queryable after self-cycle request");

    assert!(
        pane.node_items
            .iter()
            .all(|item| item != "Blend locomotion • locomotion"),
        "self-referential graph connections should not be written into the graph session"
    );
    assert!(
        !instance.dirty,
        "self-referential graph connections should remain a no-op"
    );
    assert_eq!(
        harness.runtime.editor_snapshot().status_line,
        "Ignored animation command because it did not change the current document"
    );
}

#[test]
fn animation_graph_rejects_declared_but_unimplemented_node_kinds() {
    let _guard = env_lock().lock().unwrap();
    let mut harness = EventRuntimeHarness::new("zircon_editor_event_animation_graph_unknown_kind");
    let asset_locator = open_indexed_animation_asset(
        &mut harness,
        "zircon_editor_event_animation_graph_unknown_kind_project",
        "res://animation/hero.graph.zranim",
        write_graph_asset,
    );

    let error = harness
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
                crate::core::editor_event::EditorAnimationEvent::AddGraphNode {
                    graph_locator: asset_locator.clone(),
                    node_id: "run".to_string(),
                    node_kind: "clip".to_string(),
                },
            ),
        )
        .expect_err("a declared but unavailable graph node must return a typed diagnostic");

    assert!(matches!(
        error,
        crate::ui::host::EditorEventDispatcherError::Event(
            crate::ui::host::EditorEventDispatchError::Execution(
                crate::ui::host::EditorEventExecutionError::Animation {
                    source: crate::ui::host::EditorError::AnimationCommandUnavailable {
                        diagnostic,
                    },
                }
            )
        ) if diagnostic.code() == "ZR-ANIM-CMD-002"
    ));
    assert_eq!(
        harness.runtime.editor_snapshot().status_line,
        "animation event execution failed: [ZR-ANIM-CMD-002] animation graph node kind `clip` is declared but not implemented"
    );
}

#[test]
fn animation_graph_events_reject_state_machine_documents_with_a_typed_target_diagnostic() {
    let _guard = env_lock().lock().unwrap();
    let mut harness =
        EventRuntimeHarness::new("zircon_editor_event_animation_graph_wrong_document");
    let asset_locator = open_indexed_animation_asset(
        &mut harness,
        "zircon_editor_event_animation_graph_wrong_document_project",
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
        .expect("state machine fixture must open in the shared graph view");
    let error = harness
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Animation(
                crate::core::editor_event::EditorAnimationEvent::AddGraphNode {
                    graph_locator: asset_locator,
                    node_id: "locomotion".to_string(),
                    node_kind: "blend".to_string(),
                },
            ),
        )
        .expect_err("graph events must not mutate a state-machine document");

    assert!(matches!(
        error,
        crate::ui::host::EditorEventDispatcherError::Event(
            crate::ui::host::EditorEventDispatchError::Execution(
                crate::ui::host::EditorEventExecutionError::Animation {
                    source: crate::ui::host::EditorError::AnimationTargetUnavailable {
                        diagnostic,
                    },
                }
            )
        ) if diagnostic.code() == "ZR-ANIM-TARGET-008"
    ));
}
