mod authoring;
mod capability;
mod extension_ids;
mod plugin;

pub use authoring::{
    TILEMAP_PAINT_STROKE_MAX_CELLS, TilemapEditorStats, TilemapLayerId, TilemapPaintRequest,
    TilemapPaintStrokeReceipt, apply_tilemap_paint, apply_tilemap_paint_stroke,
    supported_projection, tilemap_editor_stats, validate_tilemap_for_editor,
};
pub use capability::{CAPABILITY, EDITOR_CAPABILITIES, PLUGIN_ID};
pub use extension_ids::{TILEMAP_AUTHORING_VIEW_ID, TILEMAP_DRAWER_ID, TILEMAP_TEMPLATE_ID};
pub use plugin::{
    Tilemap2dEditorPlugin, editor_capabilities, editor_plugin, editor_plugin_descriptor,
    package_manifest, plugin_registration,
};

#[cfg(test)]
mod tests;
