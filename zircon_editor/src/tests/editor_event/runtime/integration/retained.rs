use super::super::*;

#[test]
fn retained_adapter_binding_and_call_action_share_the_same_normalized_menu_event() {
    let _guard = env_lock().lock().unwrap();

    let retained_host = EventRuntimeHarness::new("zircon_editor_event_retained_host");
    let binding = EventRuntimeHarness::new("zircon_editor_event_binding");
    let action = EventRuntimeHarness::new("zircon_editor_event_action");

    let retained_before = retained_host.runtime.editor_snapshot().scene_entries.len();
    let binding_before = binding.runtime.editor_snapshot().scene_entries.len();
    let action_before = action.runtime.editor_snapshot().scene_entries.len();

    let retained_record = retained_host
        .runtime
        .dispatch_envelope(retained_menu_action("workbench.scene.node.create.cube").unwrap())
        .unwrap();
    let binding_record = binding
        .runtime
        .dispatch_binding(
            menu_action_binding(&MenuAction::CreateNode(NodeKind::Cube)),
            EditorEventSource::Headless,
        )
        .unwrap();

    let action_response = action
        .runtime
        .handle_control_request(UiControlRequest::CallAction {
            node_path: UiNodePath::new("editor/workbench/menu/selection/scene.node.create_cube"),
            action_id: "workbench.menu.item.click".to_string(),
            arguments: Vec::new(),
        });
    let UiControlResponse::Invocation(action_result) = action_response else {
        panic!("expected invocation response");
    };
    assert_eq!(action_result.error, None);

    assert_eq!(
        retained_record.event,
        EditorEvent::WorkbenchMenu(MenuAction::CreateNode(NodeKind::Cube))
    );
    assert_eq!(binding_record.event, retained_record.event);
    assert_eq!(
        binding_record.operation_id.as_deref(),
        Some("scene.node.create_cube")
    );
    assert_eq!(
        action.runtime.journal().records()[0].event,
        retained_record.event
    );
    assert_eq!(
        action.runtime.journal().records()[0]
            .operation_id
            .as_deref(),
        Some("scene.node.create_cube")
    );
    assert_eq!(binding_record.result.value, retained_record.result.value);
    assert_eq!(action_result.value, retained_record.result.value);

    assert_eq!(
        retained_host.runtime.editor_snapshot().scene_entries.len(),
        retained_before + 1
    );
    assert_eq!(
        binding.runtime.editor_snapshot().scene_entries.len(),
        binding_before + 1
    );
    assert_eq!(
        action.runtime.editor_snapshot().scene_entries.len(),
        action_before + 1
    );

    let serialized = serde_json::to_string(&retained_record).unwrap();
    assert!(
        !serialized.contains("WorkbenchMenuBar"),
        "canonical event record leaked deleted generated UI view ids: {serialized}"
    );
}

#[test]
fn serialized_journal_replays_editor_and_layout_state_through_the_same_runtime_path() {
    let _guard = env_lock().lock().unwrap();

    let source = EventRuntimeHarness::new("zircon_editor_event_replay_source");
    source
        .runtime
        .dispatch_envelope(retained_menu_action("workbench.scene.node.create.cube").unwrap())
        .unwrap();
    source
        .runtime
        .dispatch_binding(
            EditorUiBinding::new(
                "ToolWindow",
                "AutoHideLeftTop",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::dock_command(DockCommand::SetDrawerMode {
                    slot: "left_top".to_string(),
                    mode: "AutoHide".to_string(),
                }),
            ),
            EditorEventSource::Headless,
        )
        .unwrap();

    let source_records = source.runtime.journal().records().to_vec();
    let serialized = serde_json::to_string(&source_records).unwrap();
    assert!(
        !serialized.contains("ToolWindow"),
        "journal should serialize semantic editor events instead of control ids: {serialized}"
    );

    let replay = EventRuntimeHarness::new("zircon_editor_event_replay_target");
    EditorEventReplay::replay(&replay.runtime, &source_records).unwrap();

    let source_snapshot = source.runtime.editor_snapshot();
    let replay_snapshot = replay.runtime.editor_snapshot();
    let source_layout: WorkbenchLayout = source.runtime.current_layout();
    let replay_layout: WorkbenchLayout = replay.runtime.current_layout();

    assert_eq!(
        source_snapshot.scene_entries.len(),
        replay_snapshot.scene_entries.len()
    );
    assert_eq!(
        source_snapshot
            .inspector
            .as_ref()
            .map(|inspector| inspector.name.clone()),
        replay_snapshot
            .inspector
            .as_ref()
            .map(|inspector| inspector.name.clone())
    );
    assert_eq!(source_layout, replay_layout);
    assert_eq!(
        replay.runtime.journal().records().len(),
        source_records.len()
    );
}

#[test]
fn retained_event_defers_reflection_until_the_control_request_boundary() {
    let _guard = env_lock().lock().unwrap();

    let runtime = EventRuntimeHarness::new("zircon_editor_event_deferred_reflection");
    runtime
        .runtime
        .dispatch_event(
            EditorEventSource::RetainedHost,
            EditorEvent::Transient(EditorEventTransient::HoverNode {
                node_path: "editor/workbench/pages/workbench/editor.scene#1".to_string(),
                hovered: true,
            }),
        )
        .unwrap();

    let workbench = EventViewInstanceId::new("workbench.root");
    assert!(
        runtime.runtime.context().bus().dirty_set().is_empty(),
        "an exact hover delta must not upgrade to full reflection dirtiness"
    );
    let report = runtime.runtime.drain_pending_view_refreshes();
    assert_eq!(report.deltas().node_delta_count(), 1);
    assert_eq!(report.deltas().barrier_count(), 0);
    assert!(!report.used_full_snapshot_fallback());
    assert!(report.dirty().mask_for(&workbench).is_none());

    let response = runtime
        .runtime
        .handle_control_request(UiControlRequest::QueryNode {
            node_path: UiNodePath::new("editor/workbench/pages/workbench/editor.scene#1"),
        });
    let UiControlResponse::Node(Some(scene_node)) = response else {
        panic!("expected scene node");
    };
    assert_eq!(
        scene_node.properties["transient.hovered"].reflected_value,
        json!(true)
    );
    assert!(
        runtime.runtime.context().bus().dirty_set().is_empty(),
        "control requests must materialize the latest coalesced reflection snapshot"
    );
}

#[test]
fn retained_pointer_move_burst_does_not_schedule_reflection_work() {
    let _guard = env_lock().lock().unwrap();

    let runtime = EventRuntimeHarness::new("zircon_editor_event_pointer_move_burst");
    for index in 0..1_000 {
        runtime
            .runtime
            .dispatch_event(
                EditorEventSource::RetainedHost,
                EditorEvent::Viewport(EditorViewportEvent::PointerMoved {
                    x: index as f32,
                    y: (index % 100) as f32,
                }),
            )
            .unwrap();
    }

    assert!(runtime.runtime.context().bus().dirty_set().is_empty());
    let report = runtime.runtime.drain_pending_view_refreshes();
    assert!(report.dirty().is_empty());
    assert!(report.deltas().is_empty());
    assert!(!report.used_full_snapshot_fallback());
    let journal = runtime.runtime.journal();
    assert_eq!(journal.records().len(), 1);
    assert_eq!(journal.retention_diagnostics().coalesced_records(), 999);
    assert_eq!(journal.retention_diagnostics().dropped_records(), 0);
    assert!(matches!(
        &journal.records()[0].event,
        EditorEvent::Viewport(EditorViewportEvent::PointerMoved { x, y })
            if *x == 999.0 && *y == 99.0
    ));
}

#[test]
fn retained_pointer_capture_keeps_press_release_barriers_around_coalescible_moves() {
    let _guard = env_lock().lock().unwrap();

    let runtime = EventRuntimeHarness::new("zircon_editor_event_pointer_capture_barriers");
    let press = runtime
        .runtime
        .dispatch_event(
            EditorEventSource::RetainedHost,
            EditorEvent::Viewport(EditorViewportEvent::RightPressed { x: 10.0, y: 20.0 }),
        )
        .unwrap();
    runtime
        .runtime
        .dispatch_event(
            EditorEventSource::RetainedHost,
            EditorEvent::Viewport(EditorViewportEvent::PointerMoved { x: 11.0, y: 21.0 }),
        )
        .unwrap();
    let release = runtime
        .runtime
        .dispatch_event(
            EditorEventSource::RetainedHost,
            EditorEvent::Viewport(EditorViewportEvent::RightReleased),
        )
        .unwrap();

    let report = runtime.runtime.drain_pending_view_refreshes();
    assert_eq!(report.deltas().node_delta_count(), 0);
    assert_eq!(report.deltas().barrier_count(), 2);
    assert!(matches!(
        report.deltas().entries(),
        [
            EditorUiDeltaEntry::Barrier {
                kind: EditorUiDeltaBarrierKind::Press,
                sequence,
            },
            EditorUiDeltaEntry::Barrier {
                kind: EditorUiDeltaBarrierKind::Release,
                sequence: release_sequence,
            }
        ] if *sequence == press.sequence && *release_sequence == release.sequence
    ));
    assert!(!report.used_full_snapshot_fallback());
}

#[test]
fn transient_state_projects_into_reflection_without_reading_a_live_ui_tree() {
    let _guard = env_lock().lock().unwrap();

    let runtime = EventRuntimeHarness::new("zircon_editor_event_transient");
    runtime
        .runtime
        .dispatch_event(
            EditorEventSource::RetainedHost,
            EditorEvent::Transient(EditorEventTransient::HoverNode {
                node_path: "editor/workbench/pages/workbench/editor.scene#1".to_string(),
                hovered: true,
            }),
        )
        .unwrap();
    runtime
        .runtime
        .dispatch_event(
            EditorEventSource::RetainedHost,
            EditorEvent::Transient(EditorEventTransient::FocusNode {
                node_path: "editor/workbench/pages/workbench/editor.scene#1".to_string(),
            }),
        )
        .unwrap();
    runtime
        .runtime
        .dispatch_event(
            EditorEventSource::RetainedHost,
            EditorEvent::Transient(EditorEventTransient::PressNode {
                node_path: "editor/workbench/pages/workbench/editor.scene#1".to_string(),
                pressed: true,
            }),
        )
        .unwrap();
    runtime
        .runtime
        .dispatch_event(
            EditorEventSource::RetainedHost,
            EditorEvent::Transient(EditorEventTransient::SetDrawerResizing {
                drawer_id: "left_top".to_string(),
                resizing: true,
            }),
        )
        .unwrap();

    let scene_node = runtime
        .runtime
        .handle_control_request(UiControlRequest::QueryNode {
            node_path: UiNodePath::new("editor/workbench/pages/workbench/editor.scene#1"),
        });
    let UiControlResponse::Node(Some(scene_node)) = scene_node else {
        panic!("expected scene node");
    };
    assert_eq!(
        scene_node.properties["transient.hovered"].reflected_value,
        json!(true)
    );
    assert_eq!(
        scene_node.properties["transient.focused"].reflected_value,
        json!(true)
    );
    assert!(scene_node.state_flags.pressed);

    let drawer_node = runtime
        .runtime
        .handle_control_request(UiControlRequest::QueryNode {
            node_path: UiNodePath::new("editor/workbench/drawers/left_top"),
        });
    let UiControlResponse::Node(Some(drawer_node)) = drawer_node else {
        panic!("expected drawer node");
    };
    assert_eq!(
        drawer_node.properties["transient.resizing"].reflected_value,
        json!(true)
    );
}
