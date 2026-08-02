use crate::core::asset::{AssetCreationTemplateDescriptor, AssetTypeContribution, AssetTypeId};
use crate::core::commands::EditorCommandDescriptor;
use crate::core::editor_authoring_extension::{
    GraphEditorDescriptor, GraphNodeDescriptor, GraphNodePaletteDescriptor, GraphPinDescriptor,
    SceneModeDescriptor, TimelineEditorDescriptor, TimelineTrackDescriptor,
};
use crate::core::editor_extension::{
    EditorExtensionRegistry, EditorExtensionRegistryError, EditorUiTemplateDescriptor,
};
use crate::core::editor_operation::EditorOperationPath;
use crate::core::extension::InspectorCustomizationDescriptor;

#[test]
fn authoring_descriptors_register_and_preserve_capability_gates() {
    let open = EditorOperationPath::parse("authoring.material.open").unwrap();
    let validate = EditorOperationPath::parse("authoring.material.validate").unwrap();
    let compile = EditorOperationPath::parse("authoring.material.compile").unwrap();
    let create = EditorOperationPath::parse("authoring.material.create").unwrap();
    let tool = EditorOperationPath::parse("authoring.terrain.sculpt").unwrap();
    let timeline_open = EditorOperationPath::parse("authoring.sequence.open").unwrap();
    let mut registry = EditorExtensionRegistry::default();

    for operation in [&open, &validate, &compile, &create, &tool, &timeline_open] {
        registry
            .register_command(EditorCommandDescriptor::operation(
                operation.clone(),
                operation.as_str(),
            ))
            .unwrap();
    }
    let schema_operation = EditorOperationPath::parse("authoring.material.schema_compile").unwrap();
    registry
        .register_command(
            EditorCommandDescriptor::operation(schema_operation.clone(), "Compile With Schema")
                .with_payload_schema_id("material_editor.compile_graph.v1"),
        )
        .unwrap();
    registry
        .register_asset_type_contribution(
            AssetTypeContribution::augment(AssetTypeId::parse("material.graph").unwrap())
                .with_creation_template(
                    AssetCreationTemplateDescriptor::new(
                        "material_editor.template.material_graph",
                        "Material Graph",
                        create,
                    )
                    .with_required_capabilities(["editor.extension.material_editor_authoring"]),
                ),
        )
        .unwrap();
    registry
        .register_scene_mode(crate::tests::support::pass_through_scene_mode_registration(
            SceneModeDescriptor::new(
                "terrain.tool.sculpt",
                "Sculpt Terrain",
                "terrain.authoring",
                tool,
            )
            .with_required_capabilities(["editor.extension.terrain_authoring"]),
        ))
        .unwrap();
    registry
        .register_graph_editor(
            GraphEditorDescriptor::new(
                AssetTypeId::parse("material.graph").unwrap(),
                "material_editor.graph",
                "Material Graph",
                open,
                validate,
            )
            .with_compile_operation(compile)
            .with_required_capabilities(["editor.extension.material_editor_authoring"]),
        )
        .unwrap();
    registry
        .register_graph_node_palette(
            GraphNodePaletteDescriptor::new(
                "material_editor.palette",
                AssetTypeId::parse("material.graph").unwrap(),
            )
            .with_node(
                GraphNodeDescriptor::new("output", "Output", "Material")
                    .with_input(GraphPinDescriptor::new("base_color", "vec4").required(true)),
            )
            .with_node(
                GraphNodeDescriptor::new("multiply", "Multiply", "Math")
                    .with_input(GraphPinDescriptor::new("a", "float").required(true))
                    .with_input(GraphPinDescriptor::new("b", "float").required(true))
                    .with_output(GraphPinDescriptor::new("value", "float")),
            )
            .with_required_capabilities(["editor.extension.material_editor_authoring"]),
        )
        .unwrap();
    registry
        .register_timeline_track_type(
            TimelineTrackDescriptor::new(
                "timeline_sequence.track.transform",
                "Transform",
                "transform",
            )
            .with_required_capabilities(["editor.extension.timeline_sequence_authoring"]),
        )
        .unwrap();
    registry
        .register_timeline_editor(
            TimelineEditorDescriptor::new(
                AssetTypeId::parse("animation.sequence").unwrap(),
                "timeline_sequence.timeline",
                "Timeline Sequence",
                timeline_open,
            )
            .with_track_type("timeline_sequence.track.transform")
            .with_required_capabilities(["editor.extension.timeline_sequence_authoring"]),
        )
        .unwrap();

    assert_eq!(registry.asset_type_contributions().len(), 1);
    assert_eq!(registry.scene_mode_descriptors().len(), 1);
    assert_eq!(
        registry.graph_editors()[0].asset_type().as_str(),
        "material.graph"
    );
    assert_eq!(registry.graph_node_palettes()[0].nodes().len(), 2);
    assert_eq!(
        registry.timeline_editors()[0].track_types(),
        &["timeline_sequence.track.transform".to_string()]
    );
    assert_eq!(
        registry.timeline_track_types()[0].required_capabilities(),
        &["editor.extension.timeline_sequence_authoring".to_string()]
    );
    assert_eq!(
        registry
            .pending_command(&schema_operation)
            .and_then(EditorCommandDescriptor::payload_schema_id),
        Some("material_editor.compile_graph.v1")
    );
    let schema_operation_toml = toml::to_string(
        registry
            .pending_command(&schema_operation)
            .expect("schema operation descriptor"),
    )
    .expect("operation descriptor toml");
    let decoded_schema_operation: EditorCommandDescriptor =
        toml::from_str(&schema_operation_toml).expect("operation descriptor roundtrip");
    assert_eq!(
        decoded_schema_operation.payload_schema_id(),
        Some("material_editor.compile_graph.v1")
    );
}

#[test]
fn authoring_registry_rejects_duplicate_graph_node_ids() {
    let mut registry = EditorExtensionRegistry::default();
    let error = registry
        .register_graph_node_palette(
            GraphNodePaletteDescriptor::new(
                "material_editor.palette",
                AssetTypeId::parse("material.graph").unwrap(),
            )
            .with_node(GraphNodeDescriptor::new("output", "Output", "Material"))
            .with_node(GraphNodeDescriptor::new(
                "output",
                "Duplicate Output",
                "Material",
            )),
        )
        .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("graph node output already registered")
    );
}

#[test]
fn authoring_registry_rejects_invalid_operation_payload_schema_ids() {
    let mut registry = EditorExtensionRegistry::default();
    let operation = EditorOperationPath::parse("authoring.material.compile").unwrap();
    let error = registry
        .register_command(
            EditorCommandDescriptor::operation(operation, "Compile Material")
                .with_payload_schema_id("material_editor. compile.v1"),
        )
        .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("editor command payload schema id `material_editor. compile.v1` is invalid")
    );
}

#[test]
fn authoring_registry_accepts_zui_view_templates_but_rejects_non_zui_customization_documents() {
    let mut registry = EditorExtensionRegistry::default();

    registry
        .register_ui_template(EditorUiTemplateDescriptor::new(
            "authoring.material.panel",
            "asset://material_editor/editor/panel.zui",
        ))
        .unwrap();
    registry
        .register_ui_template(EditorUiTemplateDescriptor::new(
            "authoring.material.drawer",
            "asset://material_editor/editor/drawer.zui",
        ))
        .unwrap();

    let error = registry
        .register_inspector_customization(InspectorCustomizationDescriptor::new(
            "material.Component.Graph",
            "asset://material_editor/editor/graph_drawer.toml",
            "material.GraphDrawerController",
        ))
        .unwrap_err();

    assert!(matches!(
        error,
        EditorExtensionRegistryError::InvalidUiDocument {
            kind: "inspector customization document",
            ..
        }
    ));
    assert_eq!(registry.ui_templates().len(), 2);
}
