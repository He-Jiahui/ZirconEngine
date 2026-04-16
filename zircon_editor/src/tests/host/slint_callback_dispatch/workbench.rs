use super::support::*;

#[test]
fn menu_action_dispatches_through_runtime_and_sets_scene_dirty_effects() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_callback_menu");
    let before = harness.runtime.editor_snapshot().scene_entries.len();

    let effects = dispatch_menu_action(&harness.runtime, "CreateNode.Cube").unwrap();

    assert_eq!(
        harness.runtime.editor_snapshot().scene_entries.len(),
        before + 1
    );
    assert!(effects.presentation_dirty);
    assert!(effects.render_dirty);
    assert!(!effects.layout_dirty);
    assert!(!effects.sync_asset_workspace);
}

#[test]
fn builtin_workbench_template_bridge_dispatches_reset_layout_from_shared_control_projection() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_template_bridge_reset_layout");
    let bridge = BuiltinWorkbenchTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();

    let reset_layout = bridge
        .host_projection()
        .node_by_control_id("ResetLayout")
        .expect("reset layout control should exist in builtin template projection");
    assert_eq!(reset_layout.frame, UiFrame::new(256.0, 0.0, 120.0, 32.0));

    let effects = dispatch_builtin_workbench_control(
        &harness.runtime,
        &bridge,
        "ResetLayout",
        UiEventKind::Click,
    )
    .expect("templated control should resolve")
    .unwrap();

    let journal = harness.runtime.journal();
    let record = journal.records().last().unwrap();
    assert_eq!(
        record.event,
        EditorEvent::WorkbenchMenu(crate::MenuAction::ResetLayout)
    );
    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
    assert!(!effects.render_dirty);
}

#[test]
fn builtin_workbench_open_project_requests_present_welcome_from_template_binding() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_template_bridge_open_project");
    let bridge = BuiltinWorkbenchTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();

    let effects = dispatch_builtin_workbench_menu_action(&harness.runtime, &bridge, "OpenProject")
        .expect("templated open project action should resolve")
        .unwrap();

    let journal = harness.runtime.journal();
    assert_eq!(
        journal.records().last().unwrap().event,
        EditorEvent::WorkbenchMenu(crate::MenuAction::OpenProject)
    );
    assert!(effects.present_welcome_surface);
    assert!(effects.presentation_dirty);
    assert!(!effects.layout_dirty);
    assert!(!effects.render_dirty);
    assert!(!effects.sync_asset_workspace);
}

#[test]
fn save_preset_menu_action_dispatch_updates_active_preset_name_and_status_line() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_callback_save_preset");
    let effects = dispatch_menu_action(&harness.runtime, "SavePreset.rider").unwrap();

    let journal = harness.runtime.journal();
    assert_eq!(
        journal.records().last().unwrap().event,
        EditorEvent::Layout(LayoutCommand::SavePreset {
            name: "rider".to_string(),
        })
    );
    assert_eq!(effects.active_layout_preset_name.as_deref(), Some("rider"));
    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
    assert!(!effects.render_dirty);
    assert!(!effects.reset_active_layout_preset);
    assert_eq!(
        harness.runtime.editor_snapshot().status_line,
        "Saved layout preset asset rider"
    );
}

#[test]
fn load_preset_menu_action_without_suffix_falls_back_to_current_name() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_callback_load_preset");
    dispatch_menu_action(&harness.runtime, "SavePreset.current").unwrap();
    let effects = dispatch_menu_action(&harness.runtime, "LoadPreset.").unwrap();

    let journal = harness.runtime.journal();
    assert_eq!(
        journal.records().last().unwrap().event,
        EditorEvent::Layout(LayoutCommand::LoadPreset {
            name: "current".to_string(),
        })
    );
    assert_eq!(
        effects.active_layout_preset_name.as_deref(),
        Some("current")
    );
    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
    assert!(!effects.render_dirty);
    assert!(!effects.reset_active_layout_preset);
    assert_eq!(
        harness.runtime.editor_snapshot().status_line,
        "Loaded layout preset current"
    );
}

#[test]
fn scene_menu_actions_dispatch_placeholder_status_through_callback_runtime_path() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_callback_scene_menu_actions");
    let open_effects = dispatch_menu_action(&harness.runtime, "OpenScene").unwrap();
    let create_effects = dispatch_menu_action(&harness.runtime, "CreateScene").unwrap();

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
            EditorEvent::WorkbenchMenu(crate::MenuAction::OpenScene),
            EditorEvent::WorkbenchMenu(crate::MenuAction::CreateScene),
        ]
    );
    assert!(open_effects.presentation_dirty);
    assert!(!open_effects.layout_dirty);
    assert!(!open_effects.render_dirty);
    assert!(create_effects.presentation_dirty);
    assert!(!create_effects.layout_dirty);
    assert!(!create_effects.render_dirty);
    assert_eq!(
        harness.runtime.editor_snapshot().status_line,
        "Scene open/create workflow is not wired yet"
    );
}

#[test]
fn builtin_workbench_reset_layout_matches_legacy_menu_action_dispatch() {
    let _guard = env_lock().lock().unwrap();

    let legacy_harness = EventRuntimeHarness::new("zircon_slint_parity_reset_layout_legacy");
    let legacy_effects = dispatch_menu_action(&legacy_harness.runtime, "ResetLayout").unwrap();
    let legacy_record = legacy_harness
        .runtime
        .journal()
        .records()
        .last()
        .unwrap()
        .clone();

    let builtin_harness = EventRuntimeHarness::new("zircon_slint_parity_reset_layout_builtin");
    let bridge = BuiltinWorkbenchTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();
    let builtin_effects =
        dispatch_builtin_workbench_menu_action(&builtin_harness.runtime, &bridge, "ResetLayout")
            .expect("templated reset layout action should resolve")
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
