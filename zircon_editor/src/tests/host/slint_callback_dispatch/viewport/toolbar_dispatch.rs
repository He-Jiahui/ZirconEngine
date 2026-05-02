use super::super::support::*;
use zircon_runtime_interface::ui::binding::UiBindingValue;

#[test]
fn builtin_viewport_toolbar_set_tool_dispatches_dynamic_binding_from_template() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_template_bridge_viewport_tool");
    let bridge = BuiltinViewportToolbarTemplateBridge::new().unwrap();

    let effects = dispatch_builtin_viewport_toolbar_control(
        &harness.runtime,
        &bridge,
        "SetTool",
        UiEventKind::Change,
        vec![UiBindingValue::string("Scale")],
    )
    .expect("viewport toolbar control should resolve through template bridge")
    .unwrap();

    let journal = harness.runtime.journal();
    assert_eq!(
        journal.records().last().unwrap().event,
        EditorEvent::Viewport(EditorViewportEvent::SetTool {
            tool: SceneViewportTool::Scale,
        })
    );
    assert!(effects.render_dirty);
    assert!(effects.presentation_dirty);
}

#[test]
fn builtin_viewport_toolbar_frame_selection_dispatches_static_binding_from_template() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_template_bridge_viewport_frame");
    let bridge = BuiltinViewportToolbarTemplateBridge::new().unwrap();

    let effects = dispatch_builtin_viewport_toolbar_control(
        &harness.runtime,
        &bridge,
        "FrameSelection",
        UiEventKind::Click,
        Vec::new(),
    )
    .expect("viewport toolbar frame selection should resolve through template bridge")
    .unwrap();

    let journal = harness.runtime.journal();
    assert_eq!(
        journal.records().last().unwrap().event,
        EditorEvent::Viewport(EditorViewportEvent::FrameSelection)
    );
    assert!(effects.render_dirty);
    assert!(effects.presentation_dirty);
}

#[test]
fn builtin_viewport_toolbar_play_buttons_dispatch_menu_play_mode_operations() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_template_bridge_viewport_play_mode");
    let bridge = BuiltinViewportToolbarTemplateBridge::new().unwrap();

    let enter_effects = dispatch_builtin_viewport_toolbar_control(
        &harness.runtime,
        &bridge,
        "EnterPlayMode",
        UiEventKind::Click,
        Vec::new(),
    )
    .expect("viewport toolbar play control should resolve through template bridge")
    .unwrap();
    assert_eq!(
        harness.runtime.journal().records().last().unwrap().event,
        EditorEvent::WorkbenchMenu(MenuAction::EnterPlayMode)
    );
    assert_eq!(
        harness.runtime.editor_snapshot().session_mode,
        crate::ui::workbench::startup::EditorSessionMode::Playing
    );
    assert!(enter_effects.render_dirty);
    assert!(enter_effects.presentation_dirty);

    let exit_effects = dispatch_builtin_viewport_toolbar_control(
        &harness.runtime,
        &bridge,
        "ExitPlayMode",
        UiEventKind::Click,
        Vec::new(),
    )
    .expect("viewport toolbar stop control should resolve through template bridge")
    .unwrap();
    assert_eq!(
        harness.runtime.journal().records().last().unwrap().event,
        EditorEvent::WorkbenchMenu(MenuAction::ExitPlayMode)
    );
    assert_eq!(
        harness.runtime.editor_snapshot().session_mode,
        crate::ui::workbench::startup::EditorSessionMode::Project
    );
    assert!(exit_effects.render_dirty);
    assert!(exit_effects.presentation_dirty);
}

#[test]
fn builtin_viewport_toolbar_set_tool_matches_legacy_viewport_command_dispatch() {
    let _guard = env_lock().lock().unwrap();

    let legacy_harness = EventRuntimeHarness::new("zircon_slint_parity_viewport_tool_legacy");
    let legacy_effects = dispatch_viewport_command(
        &legacy_harness.runtime,
        ViewportCommand::SetTool(SceneViewportTool::Scale),
    )
    .unwrap();
    let legacy_record = legacy_harness
        .runtime
        .journal()
        .records()
        .last()
        .unwrap()
        .clone();

    let builtin_harness = EventRuntimeHarness::new("zircon_slint_parity_viewport_tool_builtin");
    let bridge = BuiltinViewportToolbarTemplateBridge::new().unwrap();
    let builtin_effects = dispatch_builtin_viewport_toolbar_control(
        &builtin_harness.runtime,
        &bridge,
        "SetTool",
        UiEventKind::Change,
        vec![UiBindingValue::string("Scale")],
    )
    .expect("templated viewport tool control should resolve")
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
