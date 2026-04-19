use super::support::*;

#[test]
fn builtin_pane_surface_trigger_action_matches_legacy_menu_action_dispatch() {
    let _guard = env_lock().lock().unwrap();

    let legacy_harness = EventRuntimeHarness::new("zircon_slint_parity_pane_surface_action_legacy");
    let legacy_effects = dispatch_menu_action(&legacy_harness.runtime, "CreateScene").unwrap();
    let legacy_record = legacy_harness
        .runtime
        .journal()
        .records()
        .last()
        .unwrap()
        .clone();

    let builtin_harness =
        EventRuntimeHarness::new("zircon_slint_parity_pane_surface_action_builtin");
    let bridge = BuiltinPaneSurfaceTemplateBridge::new().unwrap();
    let builtin_effects = dispatch_builtin_pane_surface_control(
        &builtin_harness.runtime,
        &bridge,
        "TriggerAction",
        UiEventKind::Click,
        vec![zircon_ui::binding::UiBindingValue::string("CreateScene")],
    )
    .expect("templated pane surface action should resolve")
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
