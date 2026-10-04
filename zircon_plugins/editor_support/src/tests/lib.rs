use super::*;
use zircon_editor::core::asset::{
    AssetCreationTemplateDescriptor, AssetToolkitDescriptor, AssetTypeId, AssetTypePresentation,
    ThumbnailProviderDescriptor,
};
use zircon_editor::core::editor_authoring_extension::{
    GraphNodeDescriptor, GraphPinDescriptor, SceneModeDescriptor,
};
use zircon_editor::core::editor_message::SceneModeId;
use zircon_editor::core::editor_operation::EditorOperationPath;
use zircon_editor::scene::modes::{
    EditorSceneMode, InputOutcome, SceneModeCtx, ViewportOverlayBuilder,
};
use zircon_editor::scene::viewport::ViewportInput;

struct SupportPaintMode {
    id: SceneModeId,
}

impl EditorSceneMode for SupportPaintMode {
    fn id(&self) -> &SceneModeId {
        &self.id
    }

    fn enter(&mut self, _ctx: &mut SceneModeCtx<'_>) {}

    fn exit(&mut self, _ctx: &mut SceneModeCtx<'_>) {}

    fn handle_input(
        &mut self,
        _input: &ViewportInput,
        _ctx: &mut SceneModeCtx<'_>,
    ) -> InputOutcome {
        InputOutcome::Consumed
    }

    fn build_overlay(&self, _out: &mut ViewportOverlayBuilder) {}
}

fn operation(path: &str) -> EditorOperationPath {
    EditorOperationPath::parse(path).expect("valid test operation path")
}

#[test]
fn authoring_batch_registers_menu_items_payload_schemas_and_all_descriptor_families() {
    let import = operation("support.authoring.import");
    let open = operation("support.authoring.open");
    let validate = operation("support.authoring.validate");
    let compile = operation("support.authoring.compile");
    let create = operation("support.authoring.create");
    let activate = operation("support.authoring.activate_tool");
    let support_type = AssetTypeId::parse("support.asset").unwrap();
    let mut registry = EditorExtensionRegistry::default();

    register_authoring_contribution_batch(
        &mut registry,
        EditorAuthoringContributionBatch {
            commands: vec![
                EditorCommandDescriptor::operation(import.clone())
                    .with_payload_schema_id("support.import.v1"),
                EditorCommandDescriptor::operation(open.clone()),
                EditorCommandDescriptor::operation(validate.clone()),
                EditorCommandDescriptor::operation(compile.clone()),
                EditorCommandDescriptor::operation(create.clone()),
                EditorCommandDescriptor::operation(activate.clone()),
            ],
            menu_items: vec![EditorMenuItemDescriptor::for_operation(import.clone())
                .with_required_capabilities(["editor.extension.support_authoring"])],
            asset_importers: vec![AssetImporterDescriptor::new(
                "support.asset.importer",
                "Support Asset",
                import.clone(),
            )
            .with_source_extension("support")
            .with_output_type(support_type.clone())],
            asset_type_contributions: vec![AssetTypeContribution::define(
                support_type.clone(),
                AssetTypePresentation::new(
                    "Support Asset",
                    "SUP",
                    "asset-support",
                    "asset.support",
                ),
                ThumbnailProviderDescriptor::Icon("asset-support".to_owned()),
            )
            .with_toolkit(AssetToolkitDescriptor::new(
                "support.authoring",
                open.clone(),
            ))
            .with_creation_template(AssetCreationTemplateDescriptor::new(
                "support.template.asset",
                "Support Asset",
                create,
            ))],
            inspector_customizations: vec![InspectorCustomizationDescriptor::new(
                "support.Component",
                "plugins://support/editor/component.zui",
                "support.editor.component",
            )
            .with_binding(validate.as_str())],
            scene_modes: vec![SceneModeRegistration::new(
                SceneModeDescriptor::new(
                    "support.tool.paint",
                    "Paint Support",
                    "support.authoring",
                    activate,
                ),
                || {
                    Box::new(SupportPaintMode {
                        id: SceneModeId::new("support.tool.paint"),
                    }) as Box<dyn EditorSceneMode>
                },
            )],
            graph_editors: vec![GraphEditorDescriptor::new(
                AssetTypeId::parse("support.graph").unwrap(),
                "support.authoring",
                "Support Graph",
                open.clone(),
                validate,
            )
            .with_compile_operation(compile)],
            graph_node_palettes: vec![GraphNodePaletteDescriptor::new(
                "support.palette",
                AssetTypeId::parse("support.graph").unwrap(),
            )
            .with_node(
                GraphNodeDescriptor::new("output", "Output", "Graph")
                    .with_input(GraphPinDescriptor::new("value", "float").required(true)),
            )],
            timeline_editors: vec![TimelineEditorDescriptor::new(
                AssetTypeId::parse("support.timeline").unwrap(),
                "support.authoring",
                "Support Timeline",
                open,
            )
            .with_track_type("support.track.event")],
            timeline_track_types: vec![TimelineTrackDescriptor::new(
                "support.track.event",
                "Event",
                "event",
            )],
        },
    )
    .expect("authoring contribution batch registration");

    assert_eq!(
        registry
            .commands()
            .command(&import)
            .and_then(EditorCommandDescriptor::payload_schema_id),
        Some("support.import.v1")
    );
    let support_capabilities = vec!["editor.extension.support_authoring".to_string()];
    assert!(registry.menu_items().iter().any(|item| {
        item.path() == "support/authoring/support.authoring.import"
            && item.operation() == &import
            && item.required_capabilities() == support_capabilities.as_slice()
    }));
    assert_eq!(registry.asset_importers()[0].id(), "support.asset.importer");
    assert_eq!(
        registry.asset_type_contributions()[0].asset_type(),
        &support_type
    );
    assert_eq!(
        registry.inspector_customizations()[0].target_type(),
        "support.Component"
    );
    assert_eq!(
        registry.scene_mode_descriptors()[0].id(),
        "support.tool.paint"
    );
    assert_eq!(
        registry.graph_editors()[0].asset_type().as_str(),
        "support.graph"
    );
    assert_eq!(registry.graph_node_palettes()[0].id(), "support.palette");
    assert_eq!(
        registry.timeline_editors()[0].asset_type().as_str(),
        "support.timeline"
    );
    assert_eq!(
        registry.timeline_track_types()[0].id(),
        "support.track.event"
    );
}
