// 由视口工具栏模板触发模式和播放动作，约束模板绑定与类型化命令等价。
use super::super::support::*;
use zircon_runtime_interface::ui::binding::UiBindingValue;

#[test]
fn builtin_viewport_toolbar_activates_scene_modes_from_template() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_retained_template_bridge_viewport_tool");
    let bridge = BuiltinViewportToolbarTemplateBridge::new().unwrap();

    let effects = dispatch_builtin_viewport_toolbar_control(
        &harness.runtime,
        &bridge,
        "ActivateSceneMode",
        UiEventKind::Change,
        vec![UiBindingValue::string("Transform.Scale")],
        None,
    )
    .expect("viewport toolbar control should resolve through template bridge")
    .unwrap();

    let journal = harness.runtime.journal();
    assert_eq!(
        journal.records().last().unwrap().event,
        EditorEvent::Viewport(EditorViewportEvent::ActivateSceneMode {
            mode: SceneModeActivation::Transform(TransformHandleKind::Scale),
        })
    );
    assert!(effects.render_dirty);
    assert!(effects.sync_viewport_chrome);
    assert!(!effects.presentation_dirty);
    assert!(!effects.layout_dirty);
}

#[test]
fn builtin_viewport_toolbar_frame_selection_dispatches_static_binding_from_template() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_retained_template_bridge_viewport_frame");
    let bridge = BuiltinViewportToolbarTemplateBridge::new().unwrap();

    let effects = dispatch_builtin_viewport_toolbar_control(
        &harness.runtime,
        &bridge,
        "FrameSelection",
        UiEventKind::Click,
        Vec::new(),
        None,
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

    let harness = EventRuntimeHarness::new("zircon_retained_template_bridge_viewport_play_mode");
    let bridge = BuiltinViewportToolbarTemplateBridge::new().unwrap();

    let disabled = dispatch_builtin_viewport_toolbar_control(
        &harness.runtime,
        &bridge,
        "ExitPlayMode",
        UiEventKind::Click,
        Vec::new(),
        None,
    )
    .expect("viewport stop binding should resolve")
    .expect_err("stop must remain disabled without a live play session");
    assert!(disabled.contains("runtime.play_mode.exit is disabled by its when clause"));

    // Enter and Stop are consecutive callbacks, before a host chrome refresh.
    let enter_effects = dispatch_builtin_viewport_toolbar_control(
        &harness.runtime,
        &bridge,
        "EnterPlayMode",
        UiEventKind::Click,
        Vec::new(),
        None,
    )
    .expect("viewport toolbar play control should resolve through template bridge")
    .unwrap();
    assert_eq!(
        harness.runtime.journal().records().last().unwrap().event,
        EditorEvent::WorkbenchMenu(MenuAction::EnterPlayMode)
    );
    assert_eq!(
        harness
            .runtime
            .journal()
            .records()
            .last()
            .unwrap()
            .operation_id
            .as_deref(),
        Some("runtime.play_mode.enter")
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
        None,
    )
    .expect("viewport toolbar stop control should resolve through template bridge")
    .unwrap();
    assert_eq!(
        harness.runtime.journal().records().last().unwrap().event,
        EditorEvent::WorkbenchMenu(MenuAction::ExitPlayMode)
    );
    assert_eq!(
        harness
            .runtime
            .journal()
            .records()
            .last()
            .unwrap()
            .operation_id
            .as_deref(),
        Some("runtime.play_mode.exit")
    );
    assert_eq!(
        harness.runtime.editor_snapshot().session_mode,
        crate::ui::workbench::startup::EditorSessionMode::Project
    );
    assert!(exit_effects.render_dirty);
    assert!(exit_effects.presentation_dirty);

    let disabled = dispatch_builtin_viewport_toolbar_control(
        &harness.runtime,
        &bridge,
        "ExitPlayMode",
        UiEventKind::Click,
        Vec::new(),
        None,
    )
    .expect("viewport stop binding should resolve")
    .expect_err("stop must remain disabled without a live play session");
    assert!(disabled.contains("runtime.play_mode.exit is disabled by its when clause"));
}

#[test]
fn builtin_viewport_toolbar_mode_activation_matches_typed_command_dispatch() {
    let _guard = env_lock().lock().unwrap();

    let legacy_harness = EventRuntimeHarness::new("zircon_retained_parity_viewport_tool_legacy");
    let legacy_effects = dispatch_viewport_command(
        &legacy_harness.runtime,
        ViewportCommand::ActivateSceneMode(SceneModeActivation::Transform(
            TransformHandleKind::Scale,
        )),
    )
    .unwrap();
    let mut legacy_record = legacy_harness
        .runtime
        .journal()
        .records()
        .last()
        .unwrap()
        .clone();

    let builtin_harness = EventRuntimeHarness::new("zircon_retained_parity_viewport_tool_builtin");
    let bridge = BuiltinViewportToolbarTemplateBridge::new().unwrap();
    let builtin_effects = dispatch_builtin_viewport_toolbar_control(
        &builtin_harness.runtime,
        &bridge,
        "ActivateSceneMode",
        UiEventKind::Change,
        vec![UiBindingValue::string("Transform.Scale")],
        None,
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
    // Both paths execute the same command, while the template retains its source binding.
    assert_eq!(legacy_record.binding_path, None);
    legacy_record.binding_path = Some("ViewportToolbar/ActivateSceneMode:onChange".to_string());
    assert_eq!(builtin_record, legacy_record);
}
