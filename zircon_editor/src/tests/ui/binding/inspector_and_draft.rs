use crate::core::editor_event::InspectorFieldChange;
use crate::ui::binding::{
    DraftCommand, EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind,
};
use zircon_runtime_interface::ui::binding::UiBindingValue;

#[test]
fn inspector_batch_binding_roundtrips_with_array_payload() {
    let binding = EditorUiBinding::new(
        "InspectorView",
        "ApplyBatchButton",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::inspector_field_batch(
            "entity://selected",
            vec![
                InspectorFieldChange::new("name", UiBindingValue::string("Batch Cube")),
                InspectorFieldChange::new("transform.translation.x", UiBindingValue::Float(4.0)),
                InspectorFieldChange::new("transform.translation.y", UiBindingValue::Float(5.0)),
            ],
        ),
    );

    assert_eq!(
        binding.native_binding(),
        r#"InspectorView/ApplyBatchButton:onClick(InspectorFieldBatch("entity://selected",[["name","Batch Cube"],["transform.translation.x",4.0],["transform.translation.y",5.0]]))"#
    );
    assert_eq!(
        EditorUiBinding::parse_native_binding(&binding.native_binding()).unwrap(),
        binding
    );
}

#[test]
fn draft_command_bindings_parse_into_typed_payloads_instead_of_custom_calls() {
    let inspector = EditorUiBinding::parse_native_binding(
        r#"InspectorView/NameField:onChange(DraftCommand.SetInspectorField("entity://selected","name","Draft Cube"))"#,
    )
    .unwrap();
    assert!(
        !matches!(inspector.payload(), EditorUiBindingPayload::Custom(_)),
        "inspector draft edit should not remain a custom payload"
    );

    let mesh_import = EditorUiBinding::parse_native_binding(
        r#"AssetsView/MeshImportPathEdited:onChange(DraftCommand.SetMeshImportPath("E:/Models/cube.glb"))"#,
    )
    .unwrap();
    assert!(
        !matches!(mesh_import.payload(), EditorUiBindingPayload::Custom(_)),
        "mesh import path edit should not remain a custom payload"
    );
}

#[test]
fn inspector_draft_binding_with_arguments_rewrites_control_id_from_field_id() {
    let binding = EditorUiBinding::new(
        "InspectorView",
        "NameField",
        EditorUiEventKind::Change,
        EditorUiBindingPayload::draft_command(DraftCommand::SetInspectorField {
            subject_path: "entity://selected".to_string(),
            field_id: "name".to_string(),
            value: UiBindingValue::string(String::new()),
        }),
    );

    let rebound = binding
        .with_arguments(vec![
            UiBindingValue::string("entity://selected"),
            UiBindingValue::string("transform.translation.y"),
            UiBindingValue::string("12.5"),
        ])
        .unwrap();

    assert_eq!(rebound.path().view_id, "InspectorView");
    assert_eq!(rebound.path().control_id, "PositionYField");
    assert_eq!(
        rebound.native_binding(),
        r#"InspectorView/PositionYField:onChange(DraftCommand.SetInspectorField("entity://selected","transform.translation.y","12.5"))"#
    );
}
