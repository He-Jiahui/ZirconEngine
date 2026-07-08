use super::*;

#[test]
fn runtime_15_status_support_plan_doc_route_literals_are_child_owned() {
    let status_parent = read_runtime_src(STATUS_SUPPORT_PLAN_DOC_CHILD);
    let date_parent = read_runtime_src(DATE_SUPPORT_PLAN_DOC_CHILD);
    let status_paths = status_support_plan_doc_child_paths();
    let date_paths = date_support_plan_doc_child_paths();
    let status_children = read_sources(&status_paths).join("\n");
    let date_children = read_sources(&date_paths).join("\n");

    for moved_literal in [
        "Runtime 15 M3 status-support expected-slice map child split",
        "Runtime 15 M3 runtime index subplan map 01-15 sync",
        "Runtime 15 M3 priority plan docs code-path integrity guard",
        "Runtime 15 M3 status-support row-data owner child split",
        "Runtime 15 M3 asset-budget row-data owner child split",
        "Runtime 15 M3 mesh pipeline shader source tests child-owner split",
    ] {
        assert!(
            !status_parent.contains(moved_literal),
            "status plan_doc_support_maps.rs should delegate moved literal {moved_literal}"
        );
        assert!(
            !date_parent.contains(moved_literal),
            "date plan_doc_support_maps.rs should delegate moved literal {moved_literal}"
        );
    }

    assert_contains_all(
        "status-support plan-doc expected-slice children own moved routes",
        &format!("{status_children}\n{date_children}"),
        &[
            "Runtime 15 M3 status-support plan-doc expected-slice maps folder-backed split",
            "runtime_15_status_support_plan_doc_expected_slice_maps_folder_backed_static_passed_cargo_deferred",
            "Some(\"2026-07-05\")",
            "Runtime 15 M3 status-support expected-slice map child split",
            "Runtime 15 M3 runtime index subplan map 01-15 sync",
            "Runtime 15 M3 priority plan docs root inventory child split",
            "Runtime 15 M3 status-support row-data root inventory child split",
            "Runtime 15 M3 asset-budget row-data root inventory child split",
            "Runtime 15 M3 mesh pipeline shader source tests child-owner split",
        ],
    );
}
