use super::*;

pub(super) fn assert_status_doc_paths_rows_are_child_backed() {
    let parent = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/status_docs/paths.rs",
    );
    assert_contains_all(
        "typed-error status-doc path row parent mounts child files",
        &parent,
        &[
            "#[path = \"paths/core.rs\"]",
            "#[path = \"paths/status_current.rs\"]",
            "#[path = \"paths/child_inventory.rs\"]",
        ],
    );
    assert!(
        !parent.contains(
            "runtime_15_typed_error_status_doc_paths_child_split_static_passed_cargo_deferred"
        ),
        "status/paths.rs should route child row files instead of owning row anchors directly",
    );
    let child_blob = [
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/status_docs/paths/core.rs",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/status_docs/paths/status_current.rs",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/status_docs/paths/child_inventory.rs",
    ]
    .iter()
    .map(|path| read_runtime_src(path))
    .collect::<Vec<_>>()
    .join("\n");
    assert_contains_all(
        "typed-error status-doc path row children own representative anchors",
        &child_blob,
        &[
            "runtime_15_typed_error_status_doc_paths_child_split_static_passed_cargo_deferred",
            "runtime_15_typed_error_status_doc_paths_status_current_sources_guard_folder_backed_static_passed_cargo_deferred",
            "runtime_15_typed_error_status_doc_paths_child_inventory_split_layout_status_mirrors_status_current_folder_backed_static_passed_cargo_deferred",
        ],
    );
}
