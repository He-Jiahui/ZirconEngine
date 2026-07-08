#[path = "render_graphics/asset_font_rows.rs"]
mod asset_font_rows;
#[path = "render_graphics/expected_slice_rows.rs"]
mod expected_slice_rows;
#[path = "render_graphics/fixture_fallback_rows.rs"]
mod fixture_fallback_rows;
#[path = "render_graphics/plugin_texture_rows.rs"]
mod plugin_texture_rows;
#[path = "render_graphics/render_framework_rows.rs"]
mod render_framework_rows;
#[path = "render_graphics/scene_render_rows.rs"]
mod scene_render_rows;
#[path = "render_graphics/shader_model_rows.rs"]
mod shader_model_rows;

pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    expected_slice_rows::expected_date_for_slice(slice)
        .or_else(|| render_framework_rows::expected_date_for_slice(slice))
        .or_else(|| scene_render_rows::expected_date_for_slice(slice))
        .or_else(|| shader_model_rows::expected_date_for_slice(slice))
        .or_else(|| fixture_fallback_rows::expected_date_for_slice(slice))
        .or_else(|| plugin_texture_rows::expected_date_for_slice(slice))
        .or_else(|| asset_font_rows::expected_date_for_slice(slice))
}
