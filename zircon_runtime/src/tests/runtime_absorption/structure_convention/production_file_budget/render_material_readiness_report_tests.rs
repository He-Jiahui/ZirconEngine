use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_render_material_readiness_report_tests_are_child_owner() {
    let parent = read_runtime_src("core/framework/render/material/readiness_report.rs");
    let tests = read_runtime_src("core/framework/render/material/readiness_report/tests.rs");

    let plan_08 = read_repo("docs/plans/_archive/zircon_runtime/render/08/2026-07-09-material-shader-permutation-output-records.md");
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let material_doc = read_repo("docs/zircon_runtime/core/framework/render/material.md");

    assert_contains_all(
        "readiness report parent keeps DTO, management bridge, and child test mount",
        &parent,
        &[
            "pub struct RenderMaterialReadinessReport {",
            "pub struct RenderMaterialReadinessSummary {",
            "pub struct RenderMaterialIssueState {",
            "pub struct RenderMaterialPreparedState {",
            "pub fn management_snapshot(&self) -> RenderMaterialManagementSnapshot",
            "pub fn management_record(&self, material_id: ResourceId)",
            "pub fn push_validation_error_once(",
            "pub fn push_fallback_usage_once(",
            "pub fn push_diagnostic_once(",
            "#[cfg(test)]\nmod tests;",
        ],
    );

    for moved_test in [
        "fn material_readiness_report_deduplicates_material_uniform_diagnostics(",
        "fn material_readiness_status_classifies_issue_severity(",
        "fn material_readiness_report_summary_counts_status_and_prepared_summaries(",
    ] {
        assert!(
            !parent.contains(moved_test),
            "readiness_report.rs should mount the test child instead of defining {moved_test}"
        );
    }

    assert_contains_all(
        "readiness report test child owns readiness and management snapshot coverage",
        &tests,
        &[
            "use super::super::management::{",
            "use super::*;",
            "fn material_readiness_report_deduplicates_material_uniform_diagnostics(",
            "fn material_readiness_status_classifies_issue_severity(",
            "fn material_readiness_report_summary_counts_status_and_prepared_summaries(",
            "RenderMaterialManagementRecordSet::from_records(",
            "RenderMaterialManagementOverview::from_record_set(",
            "RenderMaterialManagementStatusView::from_records(",
        ],
    );

    for (path, source) in [
        (
            "core/framework/render/material/readiness_report.rs",
            parent.as_str(),
        ),
        (
            "core/framework/render/material/readiness_report/tests.rs",
            tests.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 material readiness owner budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Plan 08", plan_08.as_str()),
        ("render index", render_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("material docs", material_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Render material readiness report tests owner split",
                "render_plan08_material_readiness_report_tests_owner_split_static_passed_cargo_deferred_active_compile_lane",
                "core/framework/render/material/readiness_report.rs",
                "core/framework/render/material/readiness_report/tests.rs",
                "runtime_15_render_material_readiness_report_tests_are_child_owner",
            ],
        );
    }
}
