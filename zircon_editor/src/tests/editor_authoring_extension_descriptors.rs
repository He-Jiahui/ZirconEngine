use crate::core::editor_authoring_extension::{
    AssetCreationTemplateDescriptor, GraphEditorDescriptor, GraphNodeDescriptor,
    GraphNodePaletteDescriptor, GraphPinDescriptor, TimelineEditorDescriptor,
    TimelineTrackDescriptor, ViewportToolModeDescriptor,
};
use crate::core::editor_extension::EditorExtensionRegistry;
use crate::core::editor_operation::{EditorOperationDescriptor, EditorOperationPath};

#[test]
fn authoring_descriptors_register_and_preserve_capability_gates() {
    let open = EditorOperationPath::parse("Authoring.Material.Open").unwrap();
    let validate = EditorOperationPath::parse("Authoring.Material.Validate").unwrap();
    let compile = EditorOperationPath::parse("Authoring.Material.Compile").unwrap();
    let create = EditorOperationPath::parse("Authoring.Material.Create").unwrap();
    let tool = EditorOperationPath::parse("Authoring.Terrain.Sculpt").unwrap();
    let timeline_open = EditorOperationPath::parse("Authoring.Sequence.Open").unwrap();
    let mut registry = EditorExtensionRegistry::default();

    for operation in [&open, &validate, &compile, &create, &tool, &timeline_open] {
        registry
            .register_operation(EditorOperationDescriptor::new(
                operation.clone(),
                operation.as_str(),
            ))
            .unwrap();
    }
    let schema_operation = EditorOperationPath::parse("Authoring.Material.SchemaCompile").unwrap();
    registry
        .register_operation(
            EditorOperationDescriptor::new(schema_operation.clone(), "Compile With Schema")
                .with_payload_schema_id("material_editor.compile_graph.v1"),
        )
        .unwrap();
    registry
        .register_asset_creation_template(
            AssetCreationTemplateDescriptor::new(
                "material_editor.template.material_graph",
                "Material Graph",
                "material.graph",
                create,
            )
            .with_required_capabilities(["editor.extension.material_editor_authoring"]),
        )
        .unwrap();
    registry
        .register_viewport_tool_mode(
            ViewportToolModeDescriptor::new(
                "terrain.tool.sculpt",
                "Sculpt Terrain",
                "terrain.authoring",
                tool,
            )
            .with_required_capabilities(["editor.extension.terrain_authoring"]),
        )
        .unwrap();
    registry
        .register_graph_editor(
            GraphEditorDescriptor::new(
                "material.graph",
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
            GraphNodePaletteDescriptor::new("material_editor.palette", "material.graph")
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
                "animation.sequence",
                "timeline_sequence.timeline",
                "Timeline Sequence",
                timeline_open,
            )
            .with_track_type("timeline_sequence.track.transform")
            .with_required_capabilities(["editor.extension.timeline_sequence_authoring"]),
        )
        .unwrap();

    assert_eq!(registry.asset_creation_templates().len(), 1);
    assert_eq!(registry.viewport_tool_modes().len(), 1);
    assert_eq!(registry.graph_editors()[0].asset_kind(), "material.graph");
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
            .operations()
            .descriptor(&schema_operation)
            .and_then(EditorOperationDescriptor::payload_schema_id),
        Some("material_editor.compile_graph.v1")
    );
    let schema_operation_toml = toml::to_string(
        registry
            .operations()
            .descriptor(&schema_operation)
            .expect("schema operation descriptor"),
    )
    .expect("operation descriptor toml");
    let decoded_schema_operation: EditorOperationDescriptor =
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
            GraphNodePaletteDescriptor::new("material_editor.palette", "material.graph")
                .with_node(GraphNodeDescriptor::new("output", "Output", "Material"))
                .with_node(GraphNodeDescriptor::new(
                    "output",
                    "Duplicate Output",
                    "Material",
                )),
        )
        .unwrap_err();

    assert!(error
        .to_string()
        .contains("graph node output already registered"));
}

#[test]
fn authoring_registry_rejects_invalid_operation_payload_schema_ids() {
    let mut registry = EditorExtensionRegistry::default();
    let operation = EditorOperationPath::parse("Authoring.Material.Compile").unwrap();
    let error = registry
        .register_operation(
            EditorOperationDescriptor::new(operation, "Compile Material")
                .with_payload_schema_id("material_editor. compile.v1"),
        )
        .unwrap_err();

    assert!(error
        .to_string()
        .contains("operation payload schema id `material_editor. compile.v1` is invalid"));
}
