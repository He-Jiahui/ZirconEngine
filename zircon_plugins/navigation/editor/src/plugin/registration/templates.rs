use zircon_editor::core::editor_extension::{
    EditorExtensionRegistry, EditorExtensionRegistryError, EditorUiTemplateDescriptor,
};

use crate::extension_ids::{
    NAVIGATION_AGENTS_TEMPLATE_ID, NAVIGATION_ASSET_TEMPLATE_ID, NAVIGATION_BAKE_TEMPLATE_ID,
    NAVIGATION_DEBUG_TEMPLATE_ID, NAVIGATION_SETTINGS_ASSET_TEMPLATE_ID,
};

pub(super) fn register(
    registry: &mut EditorExtensionRegistry,
) -> Result<(), EditorExtensionRegistryError> {
    for (id, document) in [
        (
            NAVIGATION_AGENTS_TEMPLATE_ID,
            "plugins://navigation/editor/agents_areas.zui",
        ),
        (
            NAVIGATION_BAKE_TEMPLATE_ID,
            "plugins://navigation/editor/bake.zui",
        ),
        (
            NAVIGATION_DEBUG_TEMPLATE_ID,
            "plugins://navigation/editor/debug_gizmos.zui",
        ),
        (
            NAVIGATION_ASSET_TEMPLATE_ID,
            "plugins://navigation/editor/navmesh_asset.zui",
        ),
        (
            NAVIGATION_SETTINGS_ASSET_TEMPLATE_ID,
            "plugins://navigation/editor/navigation_settings_asset.zui",
        ),
    ] {
        registry.register_ui_template(EditorUiTemplateDescriptor::new(id, document))?;
    }
    Ok(())
}
