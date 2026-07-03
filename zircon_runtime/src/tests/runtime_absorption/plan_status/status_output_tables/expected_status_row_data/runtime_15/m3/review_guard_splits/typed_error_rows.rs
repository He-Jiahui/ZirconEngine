type Slice = super::ExpectedStatusOutputSlice;

#[path = "typed_error_rows/asset_shader_rows.rs"]
mod asset_shader_rows;
#[path = "typed_error_rows/native_plugin_rows.rs"]
mod native_plugin_rows;
#[path = "typed_error_rows/runtime_surface_rows.rs"]
mod runtime_surface_rows;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    native_plugin_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_SURFACE_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    runtime_surface_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const ASSET_SHADER_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    asset_shader_rows::EXPECTED_STATUS_OUTPUT_SLICES;

pub(super) const TYPED_ERROR_ROW_ANCHOR_MIRROR: &str = r#"
Runtime 15 M3 native plugin loader typed-error review guard child-owner split
Runtime 15 M3 native ABI surfaces typed-error review guard child-owner split
Runtime 15 M3 native plugin descriptor ABI typed-error review guard child-owner split
Runtime 15 M3 native manifest sources typed-error review guard child-owner split
Runtime 15 M3 native live-host typed-error review guard child-owner split
Runtime 15 M3 native live-host lifecycle-paths typed-error review guard child-owner split
Runtime 15 M3 native live-host replay-runtime typed-error review guard child-owner split
Runtime 15 M3 scene world typed-error review guard child-owner split
Runtime 15 M3 script host typed-error review guard child-owner split
Runtime 15 M3 UI input typed-error review guard child-owner split
Runtime 15 M3 asset loader typed-error review guard child-owner split
Runtime 15 M3 asset records typed-error review guard child-owner split
Runtime 15 M3 shader prewarm CLI typed-error review guard child-owner split
runtime_15_code_review_findings_tests_are_folder_backed
runtime_15_code_review_findings_typed_error_structure_guard_is_child_owner
Cargo gate deferred
"#;
