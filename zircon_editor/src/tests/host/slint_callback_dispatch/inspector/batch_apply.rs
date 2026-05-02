use super::super::support::*;
use zircon_runtime_interface::ui::binding::UiBindingValue;

#[test]
fn builtin_inspector_surface_apply_batch_matches_direct_batch_dispatch() {
    let _guard = env_lock().lock().unwrap();

    let batch = vec![
        InspectorFieldChange::new("name", UiBindingValue::string("Batch Cube")),
        InspectorFieldChange::new("transform.translation.x", UiBindingValue::string("4.0")),
    ];

    let legacy_harness = EventRuntimeHarness::new("zircon_slint_parity_inspector_apply_legacy");
    let legacy_effects = dispatch_inspector_apply(
        &legacy_harness.runtime,
        crate::core::editor_event::EditorInspectorEvent {
            subject_path: "entity://selected".to_string(),
            changes: batch.clone(),
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

    let builtin_harness = EventRuntimeHarness::new("zircon_slint_parity_inspector_apply_builtin");
    let bridge = BuiltinInspectorSurfaceTemplateBridge::new().unwrap();
    let builtin_effects = dispatch_builtin_inspector_surface_control(
        &builtin_harness.runtime,
        &bridge,
        "ApplyBatchButton",
        UiEventKind::Click,
        vec![
            UiBindingValue::string("entity://selected"),
            UiBindingValue::array(vec![
                UiBindingValue::array(vec![
                    UiBindingValue::string("name"),
                    UiBindingValue::string("Batch Cube"),
                ]),
                UiBindingValue::array(vec![
                    UiBindingValue::string("transform.translation.x"),
                    UiBindingValue::string("4.0"),
                ]),
            ]),
        ],
    )
    .expect("templated inspector apply control should resolve")
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
