use super::*;

#[test]
fn runtime_15_typed_error_status_doc_expected_slice_rows_are_folder_backed() {
    let status_parent = read_runtime_src(ROWS_STATUS_PARENT);
    let date_parent = read_runtime_src(ROWS_DATE_PARENT);
    let status_children = read_sources(ROWS_STATUS_CHILDREN);
    let date_children = read_sources(ROWS_DATE_CHILDREN);

    for (label, parent, function_name) in [
        (
            "status typed-error status-doc map parent",
            status_parent.as_str(),
            "expected_status_for_slice",
        ),
        (
            "date typed-error status-doc map parent",
            date_parent.as_str(),
            "expected_date_for_slice",
        ),
    ] {
        assert_contains_all(
            label,
            parent,
            &[
                "#[path = \"status_doc_rows/base_status_doc_rows.rs\"]",
                "mod base_status_doc_rows;",
                "#[path = \"status_doc_rows/paths_inventory_rows.rs\"]",
                "mod paths_inventory_rows;",
                "#[path = \"status_doc_rows/delegation_rows.rs\"]",
                "mod delegation_rows;",
                "#[path = \"status_doc_rows/status_maps_rows.rs\"]",
                "mod status_maps_rows;",
                "#[path = \"status_doc_rows/status_mirrors_rows.rs\"]",
                "mod status_mirrors_rows;",
                "#[path = \"status_doc_rows/expected_slice_map_rows.rs\"]",
                "mod expected_slice_map_rows;",
                &format!("base_status_doc_rows::{function_name}(slice)"),
                &format!("paths_inventory_rows::{function_name}(slice)"),
                &format!("delegation_rows::{function_name}(slice)"),
                &format!("status_maps_rows::{function_name}(slice)"),
                &format!("status_mirrors_rows::{function_name}(slice)"),
                &format!("expected_slice_map_rows::{function_name}(slice)"),
            ],
        );
        for moved in [
            "Runtime 15 M3 typed-error structure status-doc guard child-owner split",
            "Runtime 15 M3 typed-error status-doc paths child inventory split-layout guard folder-backed split",
            "Runtime 15 M3 typed-error status-doc delegation child split",
            "Runtime 15 M3 typed-error status-doc status mirrors child split",
        ] {
            assert!(
                !parent.contains(moved),
                "{label} should delegate moved row literal {moved}"
            );
        }
    }

    assert_contains_all(
        "typed-error status-doc status/date children",
        &format!("{status_children}\n{date_children}"),
        &[
            ROWS_SLICE,
            ROWS_STATUS,
            "Some(\"2026-07-07\")",
            "runtime_15_typed_error_structure_status_docs_child_owner_split_static_passed_cargo_deferred",
            "runtime_15_typed_error_status_doc_paths_child_inventory_split_layout_guard_folder_backed_static_passed_cargo_deferred",
            "runtime_15_typed_error_status_doc_delegation_child_split_static_passed_cargo_deferred",
            "runtime_15_typed_error_status_doc_status_maps_child_split_static_passed_cargo_deferred",
            "runtime_15_typed_error_status_doc_status_mirrors_child_split_static_passed_cargo_deferred",
        ],
    );
}

fn read_sources(paths: &[&str]) -> String {
    paths
        .iter()
        .map(|path| read_runtime_src(path))
        .collect::<Vec<_>>()
        .join("\n")
}
