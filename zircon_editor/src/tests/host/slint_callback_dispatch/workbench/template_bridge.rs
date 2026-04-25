use super::super::support::*;

#[test]
fn builtin_host_window_template_bridge_dispatches_reset_layout_from_shared_control_projection() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_template_bridge_reset_layout");
    let bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();

    let reset_layout = bridge
        .host_projection()
        .node_by_control_id("ResetLayout")
        .expect("reset layout control should exist in builtin template projection");
    assert_eq!(reset_layout.frame, UiFrame::new(256.0, 0.0, 120.0, 32.0));

    let effects =
        dispatch_builtin_host_control(&harness.runtime, &bridge, "ResetLayout", UiEventKind::Click)
            .expect("templated control should resolve")
            .unwrap();

    let journal = harness.runtime.journal();
    let record = journal.records().last().unwrap();
    assert_eq!(
        record.event,
        EditorEvent::WorkbenchMenu(MenuAction::ResetLayout)
    );
    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
    assert!(!effects.render_dirty);
}

#[test]
fn builtin_host_open_project_requests_present_welcome_from_template_binding() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_template_bridge_open_project");
    let bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();

    let effects = dispatch_builtin_host_menu_action(&harness.runtime, &bridge, "OpenProject")
        .expect("templated open project action should resolve")
        .unwrap();

    let journal = harness.runtime.journal();
    assert_eq!(
        journal.records().last().unwrap().event,
        EditorEvent::WorkbenchMenu(MenuAction::OpenProject)
    );
    assert!(effects.present_welcome_surface);
    assert!(effects.presentation_dirty);
    assert!(!effects.layout_dirty);
    assert!(!effects.render_dirty);
    assert!(!effects.sync_asset_workspace);
}

#[test]
fn builtin_host_reset_layout_matches_legacy_menu_action_dispatch() {
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
    let bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();
    let builtin_effects =
        dispatch_builtin_host_menu_action(&builtin_harness.runtime, &bridge, "ResetLayout")
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
