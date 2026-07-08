use super::*;

pub(in super::super) const STATUS_RENDER_GRAPHICS_CHILDREN: &[&str] = &[
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/naming_boundary/render_graphics/asset_font_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/naming_boundary/render_graphics/expected_slice_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/naming_boundary/render_graphics/fixture_fallback_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/naming_boundary/render_graphics/plugin_texture_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/naming_boundary/render_graphics/render_framework_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/naming_boundary/render_graphics/scene_render_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/naming_boundary/render_graphics/shader_model_rows.rs",
];

pub(in super::super) const DATE_RENDER_GRAPHICS_CHILDREN: &[&str] = &[
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/naming_boundary/render_graphics/asset_font_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/naming_boundary/render_graphics/expected_slice_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/naming_boundary/render_graphics/fixture_fallback_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/naming_boundary/render_graphics/plugin_texture_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/naming_boundary/render_graphics/render_framework_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/naming_boundary/render_graphics/scene_render_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/naming_boundary/render_graphics/shader_model_rows.rs",
];

pub(in super::super) fn read_status_naming_boundary_sources() -> String {
    read_naming_boundary_sources(STATUS_CHILD_PATHS, STATUS_RENDER_GRAPHICS_CHILDREN)
}

pub(in super::super) fn read_date_naming_boundary_sources() -> String {
    read_naming_boundary_sources(DATE_CHILD_PATHS, DATE_RENDER_GRAPHICS_CHILDREN)
}

fn read_naming_boundary_sources(parent_children: &[&str], render_children: &[&str]) -> String {
    parent_children
        .iter()
        .chain(render_children.iter())
        .map(|path| read_runtime_src(path))
        .collect::<Vec<_>>()
        .join("\n")
}
