use super::ExpectedStatusOutputSlice;

#[path = "foundation/core_rows.rs"]
mod core_rows;
#[path = "foundation/typed_error_plugin_rows.rs"]
mod typed_error_plugin_rows;
#[path = "foundation/typed_error_runtime_rows.rs"]
mod typed_error_runtime_rows;
#[path = "foundation/typed_error_scene_asset_rows.rs"]
mod typed_error_scene_asset_rows;

pub(super) const FOUNDATION_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    core_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const FOUNDATION_TYPED_ERROR_RUNTIME_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = typed_error_runtime_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const FOUNDATION_TYPED_ERROR_PLUGIN_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = typed_error_plugin_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const FOUNDATION_TYPED_ERROR_SCENE_ASSET_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = typed_error_scene_asset_rows::EXPECTED_STATUS_OUTPUT_SLICES;

pub(super) const FOUNDATION_RENDER_GRAPHICS_ROW_DATA_ANCHOR_MIRROR: &str = r#"
  ( "Runtime 15 M2 render shader definition bare-flag naming hard cutover", &[ "runtime_15_render_shader_definition_bare_flag_naming_hard_cutover_static_passed_cargo_deferred", "core/framework/render/shader/definition_value.rs", "BareFlag", "runtime_15_render_shader_definition_uses_bare_flag_names", ], ),
  ( "Runtime 15 M2 render material stale texture fixture naming hard cutover", &[ "runtime_15_render_material_stale_texture_fixture_naming_hard_cutover_static_passed_cargo_deferred", "graphics/scene/render_product_streamer_tests/material_runtime.rs", "unresolved_stale_texture", "res://textures/missing-stale-base.png", "runtime_15_render_material_stale_texture_fixtures_use_current_names", ], ),
  ( "Runtime 15 M2 UI editor showcase descriptor builders module naming hard cutover", &[ "runtime_15_ui_editor_showcase_descriptor_builders_naming_hard_cutover_static_passed_cargo_deferred", "ui/component/catalog/editor_showcase/descriptor_builders.rs", "runtime_15_ui_editor_showcase_descriptor_builders_use_owner_name", ], ),
"#;
