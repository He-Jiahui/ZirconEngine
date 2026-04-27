use zircon_editor::core::editor_event::{EditorEvent, MenuAction, ViewDescriptorId};
use zircon_editor::core::editor_extension::{
    DrawerDescriptor, EditorExtensionRegistry, EditorExtensionRegistryError,
    EditorMenuItemDescriptor, EditorUiTemplateDescriptor, ViewDescriptor,
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
