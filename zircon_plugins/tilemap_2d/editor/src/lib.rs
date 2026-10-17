#[path = "authoring.rs"]
mod authoring;
#[path = "capability.rs"]
mod capability;
#[path = "extension_ids.rs"]
mod extension_ids;
#[path = "plugin.rs"]
mod plugin;

pub use authoring::{
    apply_tilemap_paint, apply_tilemap_paint_stroke, supported_projection, tilemap_editor_stats,
    validate_tilemap_for_editor, TilemapEditorStats, TilemapLayerId, TilemapPaintRequest,
    TilemapPaintStrokeReceipt, TILEMAP_PAINT_STROKE_MAX_CELLS,
};
pub use capability::{CAPABILITY, EDITOR_CAPABILITIES, PLUGIN_ID};
pub use extension_ids::{TILEMAP_AUTHORING_VIEW_ID, TILEMAP_DRAWER_ID, TILEMAP_TEMPLATE_ID};
pub use plugin::{
    editor_capabilities, editor_plugin, editor_plugin_descriptor, package_manifest,
    plugin_registration, Tilemap2dEditorPlugin,
};

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
