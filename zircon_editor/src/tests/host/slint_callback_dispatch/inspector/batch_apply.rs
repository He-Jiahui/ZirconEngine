use super::super::support::*;

#[test]
fn builtin_inspector_surface_apply_batch_matches_direct_batch_dispatch() {
    let _guard = env_lock().lock().unwrap();

    let batch = vec![
        InspectorFieldChange::new(
            "name",
            zircon_runtime::ui::binding::UiBindingValue::string("Batch Cube"),
        ),
        InspectorFieldChange::new(
            "transform.translation.x",
            zircon_runtime::ui::binding::UiBindingValue::string("4.0"),
        ),
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
            zircon_runtime::ui::binding::UiBindingValue::string("entity://selected"),
            zircon_runtime::ui::binding::UiBindingValue::array(vec![
                zircon_runtime::ui::binding::UiBindingValue::array(vec![
                    zircon_runtime::ui::binding::UiBindingValue::string("name"),
                    zircon_runtime::ui::binding::UiBindingValue::string("Batch Cube"),
                ]),
                zircon_runtime::ui::binding::UiBindingValue::array(vec![
                    zircon_runtime::ui::binding::UiBindingValue::string("transform.translation.x"),
                    zircon_runtime::ui::binding::UiBindingValue::string("4.0"),
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
