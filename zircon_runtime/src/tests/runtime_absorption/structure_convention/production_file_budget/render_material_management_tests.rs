use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_render_material_management_tests_are_child_owners() {
    let root = read_runtime_src("core/framework/render/material/management/tests.rs");
    let record_views =
        read_runtime_src("core/framework/render/material/management/tests/record_views.rs");
    let query_execution =
        read_runtime_src("core/framework/render/material/management/tests/query_execution.rs");

    let plan_08 = read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-material-shader-permutation-output-records.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let material_doc = read_repo("docs/zircon_runtime/core/framework/render/material.md");

    assert_contains_all(
        "material management test root keeps fixtures and child mounts",
        &root,
        &[
            "fn record(",
            "fn record_with_issue_counts(",
            "mod query_execution;",
            "mod record_views;",
            "mod query_controls;",
            "mod query_result_actions;",
        ],
    );

    for moved_test in [
        "fn material_management_sort_orders_records_and_filtered_views(",
        "fn material_management_issue_summary_counts_filtered_and_selected_rows(",
        "fn material_management_issue_index_tracks_filtered_and_selected_issue_types(",
        "fn material_management_issue_view_returns_rows_for_issue_kind(",
        "fn material_management_query_filters_issue_kind_before_sorting_and_paging(",
        "fn material_management_query_selection_returns_page_details_in_display_order(",
        "fn material_management_query_filters_sorts_and_pages(",
        "fn material_management_selection_preserves_request_order_and_missing_ids(",
    ] {
        assert!(
            !root.contains(moved_test),
            "material management test root should not own moved test `{moved_test}`"
        );
    }

    assert_contains_all(
        "record view child owns sorted, summary, issue-index, and issue-view coverage",
        &record_views,
        &[
            "use super::*;",
            "fn material_management_sort_orders_records_and_filtered_views(",
            "fn material_management_issue_summary_counts_filtered_and_selected_rows(",
            "fn material_management_issue_index_tracks_filtered_and_selected_issue_types(",
            "fn material_management_issue_view_returns_rows_for_issue_kind(",
        ],
    );
    assert_contains_all(
        "query execution child owns filtering, sorting, paging, and selection coverage",
        &query_execution,
        &[
            "use super::*;",
            "fn material_management_query_filters_issue_kind_before_sorting_and_paging(",
            "fn material_management_query_selection_returns_page_details_in_display_order(",
            "fn material_management_query_filters_sorts_and_pages(",
            "fn material_management_selection_preserves_request_order_and_missing_ids(",
        ],
    );

    for (path, source) in [
        ("material/management/tests.rs", root.as_str()),
        (
            "material/management/tests/record_views.rs",
            record_views.as_str(),
        ),
        (
            "material/management/tests/query_execution.rs",
            query_execution.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay under the R4.3 test owner budget after the material management test split, got {line_count}"
        );
    }

    for (label, doc) in [
        ("Plan 08", plan_08.as_str()),
        ("render index", render_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("material docs", material_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            doc,
            &[
                "Render material management tests owner split",
                "render_plan08_material_management_tests_owner_split_static_passed_cargo_deferred_active_compile_lane",
                "core/framework/render/material/management/tests.rs",
                "core/framework/render/material/management/tests/record_views.rs",
                "core/framework/render/material/management/tests/query_execution.rs",
                "runtime_15_render_material_management_tests_are_child_owners",
            ],
        );
    }
}
