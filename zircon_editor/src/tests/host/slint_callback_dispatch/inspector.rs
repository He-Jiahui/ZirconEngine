use super::support::*;

#[test]
fn inspector_draft_field_dispatch_updates_live_snapshot_without_scene_side_effects() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_callback_inspector_draft");

    let effects =
        dispatch_inspector_draft_field(&harness.runtime, "entity://selected", "name", "Draft Cube")
            .unwrap();

    assert_eq!(
        harness
            .runtime
            .editor_snapshot()
            .inspector
            .as_ref()
            .map(|inspector| inspector.name.as_str()),
        Some("Draft Cube")
    );
    assert!(effects.presentation_dirty);
    assert!(!effects.layout_dirty);
    assert!(!effects.render_dirty);
    assert!(!effects.sync_asset_workspace);
}

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
            zircon_ui::binding::UiBindingValue::string("entity://selected"),
            zircon_ui::binding::UiBindingValue::string("name"),
            zircon_ui::binding::UiBindingValue::string("Draft Cube"),
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

#[test]
fn builtin_inspector_surface_apply_batch_matches_direct_batch_dispatch() {
    let _guard = env_lock().lock().unwrap();

    let batch = vec![
        InspectorFieldChange::new(
            "name",
            zircon_ui::binding::UiBindingValue::string("Batch Cube"),
        ),
        InspectorFieldChange::new(
            "transform.translation.x",
            zircon_ui::binding::UiBindingValue::string("4.0"),
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
            zircon_ui::binding::UiBindingValue::string("entity://selected"),
            zircon_ui::binding::UiBindingValue::array(vec![
                zircon_ui::binding::UiBindingValue::array(vec![
                    zircon_ui::binding::UiBindingValue::string("name"),
                    zircon_ui::binding::UiBindingValue::string("Batch Cube"),
                ]),
                zircon_ui::binding::UiBindingValue::array(vec![
                    zircon_ui::binding::UiBindingValue::string("transform.translation.x"),
                    zircon_ui::binding::UiBindingValue::string("4.0"),
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
