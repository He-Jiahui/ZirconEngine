type Slice = super::Slice;

#[path = "maps_guard_rows/core_rows.rs"]
mod core_rows;

const REMAINING_ROWS: [Slice; 4] = [
    (
        "Runtime 15 M3 foundation expected-slice maps guard folder-backed split",
        &[
            "runtime_15_foundation_expected_slice_maps_guard_folder_backed_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/runtime_15_expected_slice_maps/foundation.rs",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/runtime_15_expected_slice_maps/foundation/budgets.rs",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/runtime_15_expected_slice_maps/foundation/child_sources.rs",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/runtime_15_expected_slice_maps/foundation/folder_backed.rs",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/runtime_15_expected_slice_maps/foundation/paths.rs",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/runtime_15_expected_slice_maps/foundation/route_mounts.rs",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/runtime_15_expected_slice_maps/foundation/status_mirrors.rs",
            "runtime_15_foundation_expected_slice_maps_guard_is_folder_backed",
            "Cargo gate deferred",
        ],
    ),
    (
        "Runtime 15 M3 naming-boundary render-graphics map rows guard folder-backed split",
        &[
            "runtime_15_naming_boundary_render_graphics_map_rows_guard_folder_backed_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/runtime_15_expected_slice_maps/naming_boundary/render_graphics_map_rows.rs",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/runtime_15_expected_slice_maps/naming_boundary/render_graphics_map_rows/budgets.rs",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/runtime_15_expected_slice_maps/naming_boundary/render_graphics_map_rows/folder_backed.rs",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/runtime_15_expected_slice_maps/naming_boundary/render_graphics_map_rows/paths.rs",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/runtime_15_expected_slice_maps/naming_boundary/render_graphics_map_rows/route_mounts.rs",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/runtime_15_expected_slice_maps/naming_boundary/render_graphics_map_rows/status_mirrors.rs",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/runtime_15_expected_slice_maps/naming_boundary/render_graphics_map_rows/status_rows.rs",
            "runtime_15_status_output_naming_boundary_render_graphics_map_rows_guard_is_folder_backed",
            "Cargo gate deferred",
        ],
    ),
    (
        "Runtime 15 M3 foundation expected-slice maps status mirrors folder-backed split",
        &[
            "runtime_15_foundation_expected_slice_maps_status_mirrors_folder_backed_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/runtime_15_expected_slice_maps/foundation/status_mirrors.rs",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/runtime_15_expected_slice_maps/foundation/status_mirrors/budgets.rs",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/runtime_15_expected_slice_maps/foundation/status_mirrors/docs.rs",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/runtime_15_expected_slice_maps/foundation/status_mirrors/folder_backed.rs",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/runtime_15_expected_slice_maps/foundation/status_mirrors/paths.rs",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/runtime_15_expected_slice_maps/foundation/status_mirrors/row_data.rs",
            "runtime_15_foundation_expected_slice_maps_status_mirrors_are_folder_backed",
            "Cargo gate deferred",
        ],
    ),
    (
        "Runtime 15 M3 naming-boundary expected-slice sources folder-backed split",
        &[
            "runtime_15_naming_boundary_expected_slice_sources_folder_backed_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/runtime_15_expected_slice_maps/naming_boundary/sources.rs",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/runtime_15_expected_slice_maps/naming_boundary/sources/budgets.rs",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/runtime_15_expected_slice_maps/naming_boundary/sources/constants.rs",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/runtime_15_expected_slice_maps/naming_boundary/sources/folder_backed.rs",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/runtime_15_expected_slice_maps/naming_boundary/sources/guard_body.rs",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/runtime_15_expected_slice_maps/naming_boundary/sources/render_graphics.rs",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/runtime_15_expected_slice_maps/naming_boundary/sources/row_sources.rs",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/runtime_15_expected_slice_maps/naming_boundary/sources/status_mirrors.rs",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/runtime_15_expected_slice_maps/naming_boundary/sources/structure_route_maps.rs",
            "runtime_15_status_output_naming_boundary_expected_slice_sources_are_folder_backed",
            "Cargo gate deferred",
        ],
    ),
];

const COMBINED_ROWS: [Slice; 9] = [
    core_rows::ROWS[0],
    core_rows::ROWS[1],
    core_rows::ROWS[2],
    core_rows::ROWS[3],
    core_rows::ROWS[4],
    REMAINING_ROWS[0],
    REMAINING_ROWS[1],
    REMAINING_ROWS[2],
    REMAINING_ROWS[3],
];

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &COMBINED_ROWS;
