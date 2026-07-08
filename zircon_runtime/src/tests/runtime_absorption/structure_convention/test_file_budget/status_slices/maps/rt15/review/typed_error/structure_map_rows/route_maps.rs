use super::*;

#[test]
fn runtime_15_review_guard_typed_error_structure_maps_are_folder_backed() {
    let status_parent = read_runtime_src(ROWS_STATUS_PARENT);
    let date_parent = read_runtime_src(ROWS_DATE_PARENT);
    let status_children = read_sources(ROWS_STATUS_CHILDREN);
    let date_children = read_sources(ROWS_DATE_CHILDREN);

    for (label, parent, function_name) in [
        (
            "status typed-error structure map parent",
            status_parent.as_str(),
            "expected_status_for_slice",
        ),
        (
            "date typed-error structure map parent",
            date_parent.as_str(),
            "expected_date_for_slice",
        ),
    ] {
        assert_contains_all(
            label,
            parent,
            &[
                "#[path = \"typed_error_structure_maps/top_level_maps.rs\"]",
                "mod top_level_maps;",
                "#[path = \"typed_error_structure_maps/structure_guard_maps.rs\"]",
                "mod structure_guard_maps;",
                "#[path = \"typed_error_structure_maps/structure_assertion_maps.rs\"]",
                "mod structure_assertion_maps;",
                "#[path = \"typed_error_structure_maps/native_plugin_loader_maps.rs\"]",
                "mod native_plugin_loader_maps;",
                "#[path = \"typed_error_structure_maps/moved_guard_absence_maps.rs\"]",
                "mod moved_guard_absence_maps;",
                "#[path = \"typed_error_structure_maps/expected_slice_map_rows.rs\"]",
                "mod expected_slice_map_rows;",
                &format!("top_level_maps::{function_name}(slice)"),
                &format!("structure_guard_maps::{function_name}(slice)"),
                &format!("structure_assertion_maps::{function_name}(slice)"),
                &format!("native_plugin_loader_maps::{function_name}(slice)"),
                &format!("moved_guard_absence_maps::{function_name}(slice)"),
                &format!("expected_slice_map_rows::{function_name}(slice)"),
            ],
        );
        for moved in [
            "Runtime 15 M3 code review findings typed-error structure guard child-owner split",
            "Runtime 15 M3 typed-error child-ownership guard folder-backed split",
            "Runtime 15 M3 typed-error convergence mounts guard folder-backed split",
            "Runtime 15 M3 typed-error native plugin loader routes child split",
            "Runtime 15 M3 typed-error moved-guard absence guard folder-backed split",
        ] {
            assert!(
                !parent.contains(moved),
                "{label} should delegate moved typed-error structure row {moved}"
            );
        }
    }

    assert_contains_all(
        "typed-error structure status/date children",
        &format!("{status_children}\n{date_children}"),
        &[
            ROWS_SLICE,
            ROWS_STATUS,
            "Some(\"2026-07-07\")",
            "runtime_15_code_review_findings_typed_error_structure_guard_child_owner_split_static_passed_cargo_deferred",
            "runtime_15_typed_error_child_ownership_guard_folder_backed_static_passed_cargo_deferred",
            "runtime_15_typed_error_convergence_mounts_guard_folder_backed_static_passed_cargo_deferred",
            "runtime_15_typed_error_native_plugin_loader_routes_child_split_static_passed_cargo_deferred",
            "runtime_15_typed_error_moved_guard_absence_guard_folder_backed_static_passed_cargo_deferred",
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
