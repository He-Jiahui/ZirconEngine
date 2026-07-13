use crate::core::asset::AssetTypeId;
use crate::core::editor_authoring_extension::{
    GraphEditorDescriptor, GraphNodePaletteDescriptor, TimelineEditorDescriptor,
};
use crate::core::editor_extension::AssetImporterDescriptor;
use crate::core::editor_operation::EditorOperationPath;

#[test]
fn importer_graph_and_timeline_descriptors_share_the_validated_asset_type_id() {
    let material_graph = AssetTypeId::parse("material.graph").unwrap();
    let import = EditorOperationPath::parse("material.graph.import").unwrap();
    let open = EditorOperationPath::parse("material.graph.open").unwrap();
    let validate = EditorOperationPath::parse("material.graph.validate").unwrap();

    let importer = AssetImporterDescriptor::new("material.importer", "Material", import)
        .with_source_extension("zmat")
        .with_output_type(material_graph.clone());
    let graph = GraphEditorDescriptor::new(
        material_graph.clone(),
        "editor.material",
        "Material Graph",
        open.clone(),
        validate,
    );
    let palette = GraphNodePaletteDescriptor::new("material.palette", material_graph.clone());
    let timeline = TimelineEditorDescriptor::new(
        material_graph.clone(),
        "editor.material.timeline",
        "Material Timeline",
        open,
    );

    assert_eq!(importer.output_type(), Some(&material_graph));
    assert_eq!(graph.asset_type(), &material_graph);
    assert_eq!(palette.asset_type(), &material_graph);
    assert_eq!(timeline.asset_type(), &material_graph);
}

#[test]
fn descriptor_deserialization_cannot_restore_an_invalid_bare_asset_kind() {
    let invalid = r#"{
        "id":"bad.importer",
        "display_name":"Bad",
        "operation":"bad.importer.run",
        "source_extensions":["bad"],
        "output_type":"Invalid/Path",
        "priority":0,
        "required_capabilities":[]
    }"#;
    assert!(serde_json::from_str::<AssetImporterDescriptor>(invalid).is_err());
}
