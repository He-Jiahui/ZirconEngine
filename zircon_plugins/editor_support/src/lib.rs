use zircon_editor::core::editor_authoring_extension::{
    AssetCreationTemplateDescriptor, GraphEditorDescriptor, GraphNodePaletteDescriptor,
    TimelineEditorDescriptor, TimelineTrackDescriptor, ViewportToolModeDescriptor,
};
use zircon_editor::core::editor_event::{EditorEvent, MenuAction, ViewDescriptorId};
use zircon_editor::core::editor_extension::{
    AssetEditorDescriptor, AssetImporterDescriptor, ComponentDrawerDescriptor, DrawerDescriptor,
    EditorExtensionRegistry, EditorExtensionRegistryError, EditorMenuItemDescriptor,
    EditorUiTemplateDescriptor, ViewDescriptor,
};
use zircon_editor::core::editor_operation::EditorOperationDescriptor;

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
    pub operations: Vec<EditorOperationDescriptor>,
    pub asset_importers: Vec<AssetImporterDescriptor>,
    pub asset_editors: Vec<AssetEditorDescriptor>,
    pub component_drawers: Vec<ComponentDrawerDescriptor>,
    pub asset_creation_templates: Vec<AssetCreationTemplateDescriptor>,
    pub viewport_tool_modes: Vec<ViewportToolModeDescriptor>,
    pub graph_editors: Vec<GraphEditorDescriptor>,
    pub graph_node_palettes: Vec<GraphNodePaletteDescriptor>,
    pub timeline_editors: Vec<TimelineEditorDescriptor>,
    pub timeline_track_types: Vec<TimelineTrackDescriptor>,
}

pub fn register_authoring_contribution_batch(
    registry: &mut EditorExtensionRegistry,
    batch: EditorAuthoringContributionBatch,
) -> Result<(), EditorExtensionRegistryError> {
    for operation in batch.operations {
        registry.register_operation(operation)?;
    }
    for importer in batch.asset_importers {
        registry.register_asset_importer(importer)?;
    }
    for editor in batch.asset_editors {
        registry.register_asset_editor(editor)?;
    }
    for drawer in batch.component_drawers {
        registry.register_component_drawer(drawer)?;
    }
    for template in batch.asset_creation_templates {
        registry.register_asset_creation_template(template)?;
    }
    for tool_mode in batch.viewport_tool_modes {
        registry.register_viewport_tool_mode(tool_mode)?;
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
        .map_err(EditorExtensionRegistryError::Operation)?;
    registry.register_operation(
        EditorOperationDescriptor::new(
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
