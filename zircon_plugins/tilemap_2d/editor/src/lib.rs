mod authoring;
mod capability;
mod extension_ids;
mod plugin;

pub use authoring::{
    apply_tilemap_paint, supported_projection, tilemap_editor_stats, validate_tilemap_for_editor,
    TilemapEditorStats, TilemapPaintRequest,
};
pub use capability::{CAPABILITY, EDITOR_CAPABILITIES, PLUGIN_ID};
pub use extension_ids::{TILEMAP_AUTHORING_VIEW_ID, TILEMAP_DRAWER_ID, TILEMAP_TEMPLATE_ID};
pub use plugin::{
    editor_capabilities, editor_plugin, editor_plugin_descriptor, package_manifest,
    plugin_registration, Tilemap2dEditorPlugin,
};

#[cfg(test)]
mod tests;
