use super::*;

#[test]
fn runtime_15_review_guard_foundation_expected_slice_rows_are_folder_backed() {
    let status_parent = read_runtime_src(ROWS_STATUS_PARENT);
    let date_parent = read_runtime_src(ROWS_DATE_PARENT);
    let status_children = read_sources(ROWS_STATUS_CHILDREN);
    let date_children = read_sources(ROWS_DATE_CHILDREN);

    for (label, parent, function_name) in [
        (
            "status review foundation expected-slice rows parent",
            status_parent.as_str(),
            "expected_status_for_slice",
        ),
        (
            "date review foundation expected-slice rows parent",
            date_parent.as_str(),
            "expected_date_for_slice",
        ),
    ] {
        assert_contains_all(
            label,
            parent,
            &[
                "#[path = \"expected_slice_rows/route_metadata_rows.rs\"]",
                "mod route_metadata_rows;",
                "#[path = \"expected_slice_rows/root_route_rows.rs\"]",
                "mod root_route_rows;",
                "#[path = \"expected_slice_rows/foundation_status_rows.rs\"]",
                "mod foundation_status_rows;",
                "#[path = \"expected_slice_rows/source_metadata_rows.rs\"]",
                "mod source_metadata_rows;",
                "#[path = \"expected_slice_rows/expected_slice_map_rows.rs\"]",
                "mod expected_slice_map_rows;",
                &format!("route_metadata_rows::{function_name}(slice)"),
                &format!("root_route_rows::{function_name}(slice)"),
                &format!("foundation_status_rows::{function_name}(slice)"),
                &format!("source_metadata_rows::{function_name}(slice)"),
                &format!("expected_slice_map_rows::{function_name}(slice)"),
            ],
        );
        for moved in [
            "Runtime 15 M3 review-guard expected-slice route metadata child split",
            "Runtime 15 M3 review-guard expected-slice root route metadata child split",
            "Runtime 15 M3 review-guard foundation status-date maps folder-backed split",
            "Runtime 15 M3 review-guard source metadata folder-backed split",
        ] {
            assert!(
                !parent.contains(moved),
                "{label} should delegate moved review-foundation row {moved}"
            );
        }
    }

    assert_contains_all(
        "review-foundation expected-slice status/date children",
        &format!("{status_children}\n{date_children}"),
        &[
            ROWS_SLICE,
            ROWS_STATUS,
            "Some(\"2026-07-07\")",
            "runtime_15_review_guard_expected_slice_route_metadata_child_split_static_passed_cargo_deferred",
            "runtime_15_review_guard_expected_slice_root_route_metadata_child_split_static_passed_cargo_deferred",
            "runtime_15_review_guard_foundation_status_date_maps_folder_backed_static_passed_cargo_deferred",
            "runtime_15_review_guard_source_metadata_folder_backed_static_passed_cargo_deferred",
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
