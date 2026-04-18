use super::support::*;

#[test]
fn builtin_workbench_activity_toggle_collapses_active_project_drawer_from_template_binding() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_template_bridge_activity_collapse");
    let bridge = BuiltinWorkbenchTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();

    let before = harness.runtime.current_layout();
    let drawer = before.drawers.get(&ActivityDrawerSlot::LeftTop).unwrap();
    assert_eq!(drawer.mode, ActivityDrawerMode::Pinned);
    assert_eq!(
        drawer.active_view.as_ref().map(|id| id.0.as_str()),
        Some("editor.project#1")
    );

    let effects = dispatch_builtin_workbench_drawer_toggle(
        &harness.runtime,
        &bridge,
        "left_top",
        "editor.project#1",
    )
    .expect("builtin activity rail target should resolve through template bridge")
    .unwrap();

    let after = harness.runtime.current_layout();
    assert_eq!(
        after
            .drawers
            .get(&ActivityDrawerSlot::LeftTop)
            .unwrap()
            .mode,
        ActivityDrawerMode::Collapsed
    );
    let journal = harness.runtime.journal();
    assert_eq!(
        journal.records().last().unwrap().event,
        EditorEvent::Layout(LayoutCommand::SetDrawerMode {
            slot: ActivityDrawerSlot::LeftTop,
            mode: ActivityDrawerMode::Collapsed,
        })
    );
    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
}

#[test]
fn builtin_workbench_activity_toggle_reopens_collapsed_project_drawer_from_template_binding() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_template_bridge_activity_reopen");
    let bridge = BuiltinWorkbenchTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();

    let collapse = dispatch_builtin_workbench_drawer_toggle(
        &harness.runtime,
        &bridge,
        "left_top",
        "editor.project#1",
    )
    .expect("builtin activity rail target should resolve through template bridge")
    .unwrap();
    assert!(collapse.layout_dirty);

    let effects = dispatch_builtin_workbench_drawer_toggle(
        &harness.runtime,
        &bridge,
        "left_top",
        "editor.project#1",
    )
    .expect("builtin activity rail target should still resolve through template bridge")
    .unwrap();

    let after = harness.runtime.current_layout();
    let drawer = after.drawers.get(&ActivityDrawerSlot::LeftTop).unwrap();
    assert_eq!(drawer.mode, ActivityDrawerMode::Pinned);
    assert_eq!(
        drawer.active_view.as_ref().map(|id| id.0.as_str()),
        Some("editor.project#1")
    );

    let journal = harness.runtime.journal();
    let events: Vec<_> = journal
        .records()
        .iter()
        .rev()
        .take(2)
        .map(|record| record.event.clone())
        .collect();
    assert_eq!(
        events.into_iter().rev().collect::<Vec<_>>(),
        vec![
            EditorEvent::Layout(LayoutCommand::ActivateDrawerTab {
                slot: ActivityDrawerSlot::LeftTop,
                instance_id: crate::ViewInstanceId::new("editor.project#1"),
            }),
            EditorEvent::Layout(LayoutCommand::SetDrawerMode {
                slot: ActivityDrawerSlot::LeftTop,
                mode: ActivityDrawerMode::Pinned,
            }),
        ]
    );
    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
}

#[test]
fn builtin_workbench_document_tab_activation_focuses_view_from_template_binding() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_template_bridge_document_focus");
    let bridge = BuiltinWorkbenchTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();

    let document_tabs = bridge
        .host_projection()
        .node_by_control_id("DocumentTabsRoot")
        .expect("document tabs control should exist in builtin template projection");
    assert!(document_tabs.routes.iter().any(|route| {
        route.event_kind == UiEventKind::Change && route.binding_id == "DocumentTabs/ActivateTab"
    }));

    let effects = dispatch_builtin_workbench_document_tab_activation(
        &harness.runtime,
        &bridge,
        "editor.game#1",
    )
    .expect("builtin document tab activation should resolve through template bridge")
    .unwrap();

    let journal = harness.runtime.journal();
    assert_eq!(
        journal.records().last().unwrap().event,
        EditorEvent::Layout(LayoutCommand::FocusView {
            instance_id: crate::ViewInstanceId::new("editor.game#1"),
        })
    );
    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
}

#[test]
fn builtin_workbench_document_tab_close_dispatches_close_view_from_template_binding() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_template_bridge_document_close");
    let bridge = BuiltinWorkbenchTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();

    let document_tabs = bridge
        .host_projection()
        .node_by_control_id("DocumentTabsRoot")
        .expect("document tabs control should exist in builtin template projection");
    assert!(document_tabs.routes.iter().any(|route| {
        route.event_kind == UiEventKind::Submit && route.binding_id == "DocumentTabs/CloseTab"
    }));

    let effects =
        dispatch_builtin_workbench_document_tab_close(&harness.runtime, &bridge, "editor.game#1")
            .expect("builtin document tab close should resolve through template bridge")
            .unwrap();

    let journal = harness.runtime.journal();
    assert_eq!(
        journal.records().last().unwrap().event,
        EditorEvent::Layout(LayoutCommand::CloseView {
            instance_id: crate::ViewInstanceId::new("editor.game#1"),
        })
    );
    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
}

#[test]
fn builtin_workbench_host_page_activation_dispatches_activate_main_page_from_template_binding() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_template_bridge_host_page");
    let bridge = BuiltinWorkbenchTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();

    let workbench_shell = bridge
        .host_projection()
        .node_by_control_id("UiHostWindowRoot")
        .expect("workbench shell control should exist in builtin template projection");
    assert!(workbench_shell.routes.iter().any(|route| {
        route.event_kind == UiEventKind::Change
            && route.binding_id == "UiHostWindow/ActivateMainPage"
    }));

    let effects =
        dispatch_builtin_workbench_host_page_activation(&harness.runtime, &bridge, "workbench")
            .expect("builtin host page activation should resolve through template bridge")
            .unwrap();

    let journal = harness.runtime.journal();
    assert_eq!(
        journal.records().last().unwrap().event,
        EditorEvent::Layout(LayoutCommand::ActivateMainPage {
            page_id: crate::MainPageId::new("workbench"),
        })
    );
    assert!(!effects.render_dirty);
}

#[test]
fn builtin_workbench_host_page_activation_matches_legacy_layout_command_dispatch() {
    let _guard = env_lock().lock().unwrap();

    let legacy_harness = EventRuntimeHarness::new("zircon_slint_parity_host_page_legacy");
    let legacy_effects = dispatch_layout_command(
        &legacy_harness.runtime,
        LayoutCommand::ActivateMainPage {
            page_id: MainPageId::new("workbench"),
        },
    )
    .unwrap();
    let legacy_record = legacy_harness
        .runtime
        .journal()
        .records()
        .last()
        .unwrap()
        .clone();

    let builtin_harness = EventRuntimeHarness::new("zircon_slint_parity_host_page_builtin");
    let bridge = BuiltinWorkbenchTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();
    let builtin_effects = dispatch_builtin_workbench_host_page_activation(
        &builtin_harness.runtime,
        &bridge,
        "workbench",
    )
    .expect("templated host page activation should resolve")
    .unwrap();
    let builtin_record = builtin_harness
        .runtime
        .journal()
        .records()
        .last()
        .unwrap()
        .clone();

    assert_eq!(builtin_effects, legacy_effects);
    assert_eq!(builtin_record, legacy_record);
}

#[test]
fn builtin_floating_window_focus_dispatches_focus_view_from_shared_route() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_floating_window_focus");
    let window_id = MainPageId::new("window:scene");

    dispatch_layout_command(
        &harness.runtime,
        LayoutCommand::DetachViewToWindow {
            instance_id: crate::ViewInstanceId::new("editor.scene#1"),
            new_window: window_id.clone(),
        },
    )
    .unwrap();

    let effects = dispatch_builtin_floating_window_focus(&harness.runtime, &window_id)
        .expect("floating window focus should resolve through shared route")
        .unwrap();

    let journal = harness.runtime.journal();
    assert_eq!(
        journal.records().last().unwrap().event,
        EditorEvent::Layout(LayoutCommand::FocusView {
            instance_id: crate::ViewInstanceId::new("editor.scene#1"),
        })
    );
    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
}

#[test]
fn builtin_floating_window_focus_matches_legacy_layout_focus_dispatch_event_log() {
    let _guard = env_lock().lock().unwrap();

    let window_id = MainPageId::new("window:scene");

    let legacy_harness = EventRuntimeHarness::new("zircon_slint_floating_window_focus_legacy");
    dispatch_layout_command(
        &legacy_harness.runtime,
        LayoutCommand::DetachViewToWindow {
            instance_id: crate::ViewInstanceId::new("editor.scene#1"),
            new_window: window_id.clone(),
        },
    )
    .unwrap();
    let legacy_baseline = legacy_harness.runtime.journal().records().len();
    let legacy_effects = dispatch_layout_command(
        &legacy_harness.runtime,
        LayoutCommand::FocusView {
            instance_id: crate::ViewInstanceId::new("editor.scene#1"),
        },
    )
    .unwrap();
    let legacy_snapshot = crate::EditorUiCompatibilityHarness::capture_event_journal_delta_snapshot(
        &legacy_harness.runtime.journal(),
        legacy_baseline,
    );

    let builtin_harness = EventRuntimeHarness::new("zircon_slint_floating_window_focus_builtin");
    dispatch_layout_command(
        &builtin_harness.runtime,
        LayoutCommand::DetachViewToWindow {
            instance_id: crate::ViewInstanceId::new("editor.scene#1"),
            new_window: window_id.clone(),
        },
    )
    .unwrap();
    let builtin_baseline = builtin_harness.runtime.journal().records().len();
    let builtin_effects =
        dispatch_builtin_floating_window_focus(&builtin_harness.runtime, &window_id)
            .expect("floating window focus should resolve through shared route")
            .unwrap();
    let builtin_snapshot =
        crate::EditorUiCompatibilityHarness::capture_event_journal_delta_snapshot(
            &builtin_harness.runtime.journal(),
            builtin_baseline,
        );

    assert_eq!(builtin_effects, legacy_effects);
    assert_eq!(
        builtin_snapshot.event_entries,
        legacy_snapshot.event_entries
    );
}

#[test]
fn builtin_floating_window_focus_for_source_dispatches_when_switching_windows() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_floating_window_focus_source");
    let window_id = MainPageId::new("window:scene");

    dispatch_layout_command(
        &harness.runtime,
        LayoutCommand::DetachViewToWindow {
            instance_id: crate::ViewInstanceId::new("editor.scene#1"),
            new_window: window_id.clone(),
        },
    )
    .unwrap();

    let effects =
        dispatch_builtin_floating_window_focus_for_source(&harness.runtime, Some(&window_id), None)
            .expect("child window source should trigger focus dispatch")
            .unwrap();

    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
    assert_eq!(
        harness.runtime.journal().records().last().unwrap().event,
        EditorEvent::Layout(LayoutCommand::FocusView {
            instance_id: crate::ViewInstanceId::new("editor.scene#1"),
        })
    );
}

#[test]
fn builtin_floating_window_focus_for_source_skips_redundant_focus_for_same_window() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_floating_window_focus_source_skip");
    let window_id = MainPageId::new("window:scene");

    dispatch_layout_command(
        &harness.runtime,
        LayoutCommand::DetachViewToWindow {
            instance_id: crate::ViewInstanceId::new("editor.scene#1"),
            new_window: window_id.clone(),
        },
    )
    .unwrap();
    let baseline = harness.runtime.journal().records().len();

    assert!(
        dispatch_builtin_floating_window_focus_for_source(
            &harness.runtime,
            Some(&window_id),
            Some(&window_id),
        )
        .is_none(),
        "same floating callback source should not emit another focus dispatch"
    );
    assert_eq!(harness.runtime.journal().records().len(), baseline);
}

#[test]
fn tab_drop_dispatch_attaches_view_and_reopens_target_drawer_from_normalized_route() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_tab_drop_dispatch");
    dispatch_layout_command(
        &harness.runtime,
        LayoutCommand::SetDrawerMode {
            slot: ActivityDrawerSlot::RightTop,
            mode: ActivityDrawerMode::Collapsed,
        },
    )
    .unwrap();

    let effects = dispatch_tab_drop(
        &harness.runtime,
        "editor.project#1",
        &ResolvedWorkbenchTabDropRoute {
            target_group: WorkbenchDragTargetGroup::Right,
            target_label: "right tool stack",
            target: ResolvedWorkbenchTabDropTarget::Attach(ResolvedTabDrop {
                host: ViewHost::Drawer(ActivityDrawerSlot::RightTop),
                anchor: None,
            }),
        },
    )
    .unwrap();

    let journal = harness.runtime.journal();
    let events: Vec<_> = journal
        .records()
        .iter()
        .rev()
        .take(2)
        .map(|record| record.event.clone())
        .collect();
    assert_eq!(
        events.into_iter().rev().collect::<Vec<_>>(),
        vec![
            EditorEvent::Layout(LayoutCommand::AttachView {
                instance_id: ViewInstanceId::new("editor.project#1"),
                target: ViewHost::Drawer(ActivityDrawerSlot::RightTop),
                anchor: None,
            }),
            EditorEvent::Layout(LayoutCommand::SetDrawerMode {
                slot: ActivityDrawerSlot::RightTop,
                mode: ActivityDrawerMode::Pinned,
            }),
        ]
    );
    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
}

#[test]
fn tab_drop_dispatch_does_not_reopen_drawer_when_target_is_already_visible() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_tab_drop_attach_visible");

    let baseline_records = harness.runtime.journal().records().len();
    let effects = dispatch_tab_drop(
        &harness.runtime,
        "editor.project#1",
        &ResolvedWorkbenchTabDropRoute {
            target_group: WorkbenchDragTargetGroup::Right,
            target_label: "right tool stack",
            target: ResolvedWorkbenchTabDropTarget::Attach(ResolvedTabDrop {
                host: ViewHost::Drawer(ActivityDrawerSlot::RightTop),
                anchor: None,
            }),
        },
    )
    .unwrap();

    let journal = harness.runtime.journal();
    let new_records = &journal.records()[baseline_records..];
    assert_eq!(new_records.len(), 1);
    assert_eq!(
        new_records[0].event,
        EditorEvent::Layout(LayoutCommand::AttachView {
            instance_id: ViewInstanceId::new("editor.project#1"),
            target: ViewHost::Drawer(ActivityDrawerSlot::RightTop),
            anchor: None,
        })
    );
    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
}

#[test]
fn tab_drop_dispatch_preserves_auto_hide_drawer_mode() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_tab_drop_attach_autohide");
    dispatch_layout_command(
        &harness.runtime,
        LayoutCommand::SetDrawerMode {
            slot: ActivityDrawerSlot::RightTop,
            mode: ActivityDrawerMode::AutoHide,
        },
    )
    .unwrap();

    let baseline_records = harness.runtime.journal().records().len();
    let effects = dispatch_tab_drop(
        &harness.runtime,
        "editor.project#1",
        &ResolvedWorkbenchTabDropRoute {
            target_group: WorkbenchDragTargetGroup::Right,
            target_label: "right tool stack",
            target: ResolvedWorkbenchTabDropTarget::Attach(ResolvedTabDrop {
                host: ViewHost::Drawer(ActivityDrawerSlot::RightTop),
                anchor: None,
            }),
        },
    )
    .unwrap();

    let journal = harness.runtime.journal();
    let new_records = &journal.records()[baseline_records..];
    assert_eq!(new_records.len(), 1);
    assert_eq!(
        new_records[0].event,
        EditorEvent::Layout(LayoutCommand::AttachView {
            instance_id: ViewInstanceId::new("editor.project#1"),
            target: ViewHost::Drawer(ActivityDrawerSlot::RightTop),
            anchor: None,
        })
    );
    assert_eq!(
        harness
            .runtime
            .current_layout()
            .drawers
            .get(&ActivityDrawerSlot::RightTop)
            .map(|drawer| drawer.mode),
        Some(ActivityDrawerMode::AutoHide)
    );
    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
}

#[test]
fn tab_drop_dispatch_creates_split_for_document_edge_route() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_tab_drop_split_dispatch");

    let effects = dispatch_tab_drop(
        &harness.runtime,
        "editor.project#1",
        &ResolvedWorkbenchTabDropRoute {
            target_group: WorkbenchDragTargetGroup::Document,
            target_label: "Split Document Left",
            target: ResolvedWorkbenchTabDropTarget::Split {
                workspace: WorkspaceTarget::MainPage(MainPageId::workbench()),
                path: vec![],
                axis: SplitAxis::Horizontal,
                placement: SplitPlacement::Before,
            },
        },
    )
    .unwrap();

    let journal = harness.runtime.journal();
    assert_eq!(
        journal.records().last().unwrap().event,
        EditorEvent::Layout(LayoutCommand::CreateSplit {
            workspace: WorkspaceTarget::MainPage(MainPageId::workbench()),
            path: vec![],
            axis: SplitAxis::Horizontal,
            placement: SplitPlacement::Before,
            new_instance: ViewInstanceId::new("editor.project#1"),
        })
    );
    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
}
