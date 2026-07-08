use super::*;
#[rustfmt::skip]
const FOUNDATION_CHILD_STEMS: &[&str] = &["asset_provider_cleanup", "core_cleanup", "graphics_diagnostics", "lock_poison", "map_rows", "typed_error_core", "typed_error_plugin"];
#[rustfmt::skip]
const RENDER_CHILD_STEMS: &[&str] = &["asset_font_rows", "expected_slice_rows", "fixture_fallback_rows", "plugin_texture_rows", "render_framework_rows", "scene_render_rows", "shader_model_rows"];
#[rustfmt::skip]
const M3_M4_CHILD_STEMS: &[&str] = &["expected_slice_guard_maps", "m3_row_data_maps", "m4_row_data_maps", "status_support_guard_maps"];
pub(super) fn joined_runtime_sources(paths: &[&str]) -> String {
    let mut sources = Vec::new();
    for path in paths {
        sources.push(read_runtime_src(path));
        append_child_sources(&mut sources, path);
    }
    sources.join("\n")
}
fn append_child_sources(sources: &mut Vec<String>, path: &str) {
    let child_dir = path.trim_end_matches(".rs");
    let child_stems = if path.ends_with("runtime_15/foundation.rs") {
        FOUNDATION_CHILD_STEMS
    } else if path.ends_with("naming_boundary/render_graphics.rs") {
        RENDER_CHILD_STEMS
    } else if path.ends_with("status_support_maps/m3_m4_expected_slice_maps.rs") {
        M3_M4_CHILD_STEMS
    } else {
        &[]
    };
    for child in child_stems {
        sources.push(read_runtime_src(&format!("{child_dir}/{child}.rs")));
    }
}
pub(super) fn read_child_owner(relative_path: &str) -> String {
    read_runtime_src(&format!(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/owners/{relative_path}"
    ))
}
