#[path = "naming_boundary/core_bootstrap.rs"]
mod core_bootstrap;
#[path = "naming_boundary/plugin_ui_platform.rs"]
mod plugin_ui_platform;
#[path = "naming_boundary/render_graphics.rs"]
mod render_graphics;
#[path = "naming_boundary/scene_asset_runtime.rs"]
mod scene_asset_runtime;

// Runtime 15 M2 render/material naming-boundary anchor mirror for direct
// date-slice text guards:
// Runtime 15 M2 render material stale texture fixture naming hard cutover
// runtime_15_render_material_stale_texture_fixture_naming_hard_cutover_static_passed_cargo_deferred
// graphics/scene/render_product_streamer_tests/material_runtime.rs
// unresolved_stale_texture
// runtime_15_render_material_stale_texture_fixtures_use_current_names
// Runtime 15 M2 material asset schema-v1 defaults naming hard cutover
// runtime_15_material_asset_schema_v1_defaults_naming_hard_cutover_static_passed_cargo_deferred
// asset/assets/material/material_asset.rs
// property_overrides_with_schema_v1_defaults
// naming_boundary/runtime_15_m2/asset_schema.rs
// runtime_15_material_asset_schema_v1_defaults_use_versioned_names

pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    core_bootstrap::expected_date_for_slice(slice)
        .or_else(|| scene_asset_runtime::expected_date_for_slice(slice))
        .or_else(|| plugin_ui_platform::expected_date_for_slice(slice))
        .or_else(|| render_graphics::expected_date_for_slice(slice))
}
