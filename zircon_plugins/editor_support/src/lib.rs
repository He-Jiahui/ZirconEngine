use zircon_editor::core::asset::AssetTypeContribution;
use zircon_editor::core::commands::EditorCommandDescriptor;
use zircon_editor::core::editor_authoring_extension::{
    GraphEditorDescriptor, GraphNodePaletteDescriptor, TimelineEditorDescriptor,
    TimelineTrackDescriptor,
};
use zircon_editor::core::editor_event::{EditorEvent, MenuAction, ViewDescriptorId};
use zircon_editor::core::editor_extension::{
    AssetImporterDescriptor, ComponentDrawerDescriptor, DrawerDescriptor, EditorExtensionRegistry,
    EditorExtensionRegistryError, EditorMenuItemDescriptor, EditorUiTemplateDescriptor,
    ViewDescriptor,
};
use zircon_editor::scene::modes::SceneModeRegistration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EditorAuthoringSurface<'a> {
    pub view_id: &'a str,
    pub display_name: &'a str,
    pub category: &'a str,
    pub menu_path: &'a str,
}

impl<'a> EditorAuthoringSurface<'a> {
    pub const fn new(
        view_id: &'a str,
        display_name: &'a str,
        category: &'a str,
        menu_path: &'a str,
    ) -> Self {
        Self {
            view_id,
            display_name,
            category,
            menu_path,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EditorAuthoringExtensions<'a> {
    pub drawer_id: &'a str,
    pub drawer_display_name: &'a str,
    pub template_id: &'a str,
    pub template_document: &'a str,
    pub surfaces: &'a [EditorAuthoringSurface<'a>],
}

pub fn register_authoring_extensions(
    registry: &mut EditorExtensionRegistry,
    extensions: EditorAuthoringExtensions<'_>,
) -> Result<(), EditorExtensionRegistryError> {
    registry.register_drawer(DrawerDescriptor::new(
        extensions.drawer_id,
        extensions.drawer_display_name,
    ))?;
    registry.register_ui_template(EditorUiTemplateDescriptor::new(
        extensions.template_id,
        extensions.template_document,
    ))?;
    for surface in extensions.surfaces {
        register_authoring_surface(registry, *surface)?;
    }
    Ok(())
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct EditorAuthoringContributionBatch {
    pub commands: Vec<EditorCommandDescriptor>,
    pub menu_items: Vec<EditorMenuItemDescriptor>,
    pub asset_importers: Vec<AssetImporterDescriptor>,
    pub asset_type_contributions: Vec<AssetTypeContribution>,
    pub component_drawers: Vec<ComponentDrawerDescriptor>,
    pub scene_modes: Vec<SceneModeRegistration>,
    pub graph_editors: Vec<GraphEditorDescriptor>,
    pub graph_node_palettes: Vec<GraphNodePaletteDescriptor>,
    pub timeline_editors: Vec<TimelineEditorDescriptor>,
    pub timeline_track_types: Vec<TimelineTrackDescriptor>,
}

pub fn register_authoring_contribution_batch(
    registry: &mut EditorExtensionRegistry,
    batch: EditorAuthoringContributionBatch,
) -> Result<(), EditorExtensionRegistryError> {
    for operation in batch.commands {
        registry.register_command(operation)?;
    }
    for menu_item in batch.menu_items {
        registry.register_menu_item(menu_item)?;
    }
    for importer in batch.asset_importers {
        registry.register_asset_importer(importer)?;
    }
    for contribution in batch.asset_type_contributions {
        registry.register_asset_type_contribution(contribution)?;
    }
    for drawer in batch.component_drawers {
        registry.register_component_drawer(drawer)?;
    }
    for scene_mode in batch.scene_modes {
        registry.register_scene_mode(scene_mode)?;
    }
    for graph_editor in batch.graph_editors {
        registry.register_graph_editor(graph_editor)?;
    }
    for palette in batch.graph_node_palettes {
        registry.register_graph_node_palette(palette)?;
    }
    for editor in batch.timeline_editors {
        registry.register_timeline_editor(editor)?;
    }
    for track_type in batch.timeline_track_types {
        registry.register_timeline_track_type(track_type)?;
    }
    Ok(())
}

pub fn register_authoring_surface(
    registry: &mut EditorExtensionRegistry,
    surface: EditorAuthoringSurface<'_>,
) -> Result<(), EditorExtensionRegistryError> {
    let view = ViewDescriptor::new(surface.view_id, surface.display_name, surface.category);
    let operation_path = view
        .open_operation_path()
        .map_err(EditorExtensionRegistryError::OperationPath)?;
    registry.register_command(
        EditorCommandDescriptor::operation(
            operation_path.clone(),
            format!("Open {}", view.display_name()),
        )
        .with_menu_path(surface.menu_path)
        .with_event(EditorEvent::WorkbenchMenu(MenuAction::OpenView(
            ViewDescriptorId::new(view.id()),
        ))),
    )?;
    registry.register_menu_item(EditorMenuItemDescriptor::new(
        surface.menu_path,
        operation_path,
    ))?;
    registry.register_view(view)
}

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_editor::core::asset::{
        AssetCreationTemplateDescriptor, AssetToolkitDescriptor, AssetTypeId,
        AssetTypePresentation, ThumbnailProviderDescriptor,
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
                    EditorCommandDescriptor::operation(import.clone(), "Import Support Asset")
                        .with_menu_path("Plugins/Support/Import")
                        .with_payload_schema_id("support.import.v1"),
                    EditorCommandDescriptor::operation(open.clone(), "Open Support Asset"),
                    EditorCommandDescriptor::operation(validate.clone(), "Validate Support Asset"),
                    EditorCommandDescriptor::operation(compile.clone(), "Compile Support Asset"),
                    EditorCommandDescriptor::operation(create.clone(), "Create Support Asset"),
                    EditorCommandDescriptor::operation(activate.clone(), "Activate Support Tool"),
                ],
                menu_items: vec![
                    EditorMenuItemDescriptor::new("Plugins/Support/Import", import.clone())
                        .with_required_capabilities(["editor.extension.support_authoring"]),
                ],
                asset_importers: vec![
                    AssetImporterDescriptor::new(
                        "support.asset.importer",
                        "Support Asset",
                        import.clone(),
                    )
                    .with_source_extension("support")
                    .with_output_type(support_type.clone()),
                ],
                asset_type_contributions: vec![
                    AssetTypeContribution::define(
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
                    .with_creation_template(
                        AssetCreationTemplateDescriptor::new(
                            "support.template.asset",
                            "Support Asset",
                            create,
                        ),
                    ),
                ],
                component_drawers: vec![
                    ComponentDrawerDescriptor::new(
                        "support.Component",
                        "plugins://support/editor/component.zui",
                        "support.editor.component",
                    )
                    .with_binding(validate.as_str()),
                ],
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
                graph_editors: vec![
                    GraphEditorDescriptor::new(
                        AssetTypeId::parse("support.graph").unwrap(),
                        "support.authoring",
                        "Support Graph",
                        open.clone(),
                        validate,
                    )
                    .with_compile_operation(compile),
                ],
                graph_node_palettes: vec![
                    GraphNodePaletteDescriptor::new(
                        "support.palette",
                        AssetTypeId::parse("support.graph").unwrap(),
                    )
                    .with_node(
                        GraphNodeDescriptor::new("output", "Output", "Graph")
                            .with_input(GraphPinDescriptor::new("value", "float").required(true)),
                    ),
                ],
                timeline_editors: vec![
                    TimelineEditorDescriptor::new(
                        AssetTypeId::parse("support.timeline").unwrap(),
                        "support.authoring",
                        "Support Timeline",
                        open,
                    )
                    .with_track_type("support.track.event"),
                ],
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
            item.path() == "Plugins/Support/Import"
                && item.operation() == &import
                && item.required_capabilities() == support_capabilities.as_slice()
        }));
        assert_eq!(registry.asset_importers()[0].id(), "support.asset.importer");
        assert_eq!(
            registry.asset_type_contributions()[0].asset_type(),
            &support_type
        );
        assert_eq!(
            registry.component_drawers()[0].component_type(),
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
}
