use zircon_editor::core::asset::AssetTypeContribution;
use zircon_editor::core::commands::{EditorCommandDescriptor, EditorCommandMenuPath};
use zircon_editor::core::editor_authoring_extension::{
    GraphEditorDescriptor, GraphNodePaletteDescriptor, TimelineEditorDescriptor,
    TimelineTrackDescriptor,
};
use zircon_editor::core::editor_event::{EditorEvent, MenuAction, ViewDescriptorId};
use zircon_editor::core::editor_extension::{
    AssetImporterDescriptor, DrawerDescriptor, EditorExtensionRegistry,
    EditorExtensionRegistryError, EditorMenuItemDescriptor, EditorUiTemplateDescriptor,
    ViewDescriptor,
};
use zircon_editor::core::extension::InspectorCustomizationDescriptor;
use zircon_editor::scene::modes::SceneModeRegistration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EditorAuthoringSurface<'a> {
    pub view_id: &'a str,
    pub display_name: &'a str,
    pub category: &'a str,
}

impl<'a> EditorAuthoringSurface<'a> {
    pub const fn new(view_id: &'a str, display_name: &'a str, category: &'a str) -> Self {
        Self {
            view_id,
            display_name,
            category,
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
    pub inspector_customizations: Vec<InspectorCustomizationDescriptor>,
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
    for customization in batch.inspector_customizations {
        registry.register_inspector_customization(customization)?;
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
        EditorCommandDescriptor::operation(operation_path.clone())
            .with_menu_path(EditorCommandMenuPath::builtin(
                &operation_path,
                "view",
                &["plugins"],
            ))
            .with_event(EditorEvent::WorkbenchMenu(MenuAction::OpenView(
                ViewDescriptorId::new(view.id()),
            ))),
    )?;
    registry.register_view(view)
}

#[cfg(test)]
#[path = "tests/lib.rs"]
mod tests;
