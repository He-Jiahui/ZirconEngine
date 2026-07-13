mod assets;
mod components;
mod operations;
mod templates;

use zircon_editor::core::editor_extension::{
    EditorExtensionRegistry, EditorExtensionRegistryError,
};
use zircon_plugin_editor_support::{
    register_authoring_extensions, EditorAuthoringExtensions, EditorAuthoringSurface,
};

use crate::extension_ids::{
    NAVIGATION_AGENTS_VIEW_ID, NAVIGATION_AUTHORING_VIEW_ID, NAVIGATION_BAKE_VIEW_ID,
    NAVIGATION_DEBUG_VIEW_ID, NAVIGATION_DRAWER_ID, NAVIGATION_TEMPLATE_ID,
};
use crate::overlay::register_navigation_overlay;

pub(crate) fn register_navigation_extensions(
    registry: &mut EditorExtensionRegistry,
) -> Result<(), EditorExtensionRegistryError> {
    register_authoring_extensions(
        registry,
        EditorAuthoringExtensions {
            drawer_id: NAVIGATION_DRAWER_ID,
            drawer_display_name: "Navigation Tools",
            template_id: NAVIGATION_TEMPLATE_ID,
            template_document: "plugins://navigation/editor/surfaces.zui",
            surfaces: &[
                EditorAuthoringSurface::new(
                    NAVIGATION_AUTHORING_VIEW_ID,
                    "Navigation Surfaces",
                    "World",
                    "Plugins/Navigation/Surfaces",
                ),
                EditorAuthoringSurface::new(
                    NAVIGATION_AGENTS_VIEW_ID,
                    "Navigation Agents & Areas",
                    "World",
                    "Plugins/Navigation/Agents & Areas",
                ),
                EditorAuthoringSurface::new(
                    NAVIGATION_BAKE_VIEW_ID,
                    "Navigation Bake",
                    "World",
                    "Plugins/Navigation/Bake",
                ),
                EditorAuthoringSurface::new(
                    NAVIGATION_DEBUG_VIEW_ID,
                    "Navigation Debug",
                    "World",
                    "Plugins/Navigation/Debug",
                ),
            ],
        },
    )?;
    templates::register(registry)?;
    components::register(registry)?;
    operations::register(registry)?;
    register_navigation_overlay(registry)?;
    assets::register(registry)
}
