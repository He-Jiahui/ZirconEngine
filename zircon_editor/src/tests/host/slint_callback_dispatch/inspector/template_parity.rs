use super::super::support::*;
use zircon_runtime_interface::ui::binding::UiBindingValue;

#[test]
fn builtin_inspector_surface_name_field_matches_direct_binding_dispatch() {
    let _guard = env_lock().lock().unwrap();

    let legacy_harness = EventRuntimeHarness::new("zircon_slint_parity_inspector_name_legacy");
    let legacy_effects = dispatch_inspector_draft_field(
        &legacy_harness.runtime,
        "entity://selected",
        "name",
        "Draft Cube",
    )
    .unwrap();
    let legacy_record = legacy_harness
        .runtime
        .journal()
        .records()
        .last()
        .unwrap()
        .clone();

    let builtin_harness = EventRuntimeHarness::new("zircon_slint_parity_inspector_name_builtin");
    let bridge = BuiltinInspectorSurfaceTemplateBridge::new().unwrap();
    let builtin_effects = dispatch_builtin_inspector_surface_control(
        &builtin_harness.runtime,
        &bridge,
        "NameField",
        UiEventKind::Change,
        vec![
            UiBindingValue::string("entity://selected"),
            UiBindingValue::string("name"),
            UiBindingValue::string("Draft Cube"),
        ],
    )
    .expect("templated inspector name control should resolve")
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
fn builtin_inspector_surface_delete_selected_matches_legacy_binding_dispatch() {
    let _guard = env_lock().lock().unwrap();

    let legacy_harness = EventRuntimeHarness::new("zircon_slint_parity_inspector_delete_legacy");
    let legacy_effects = dispatch_inspector_delete_selected(&legacy_harness.runtime).unwrap();
    let legacy_record = legacy_harness
        .runtime
        .journal()
        .records()
        .last()
        .unwrap()
        .clone();

    let builtin_harness = EventRuntimeHarness::new("zircon_slint_parity_inspector_delete_builtin");
    let bridge = BuiltinInspectorSurfaceTemplateBridge::new().unwrap();
    let builtin_effects = dispatch_builtin_inspector_surface_control(
        &builtin_harness.runtime,
        &bridge,
        "DeleteSelected",
        UiEventKind::Click,
        Vec::new(),
    )
    .expect("templated inspector delete control should resolve")
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
