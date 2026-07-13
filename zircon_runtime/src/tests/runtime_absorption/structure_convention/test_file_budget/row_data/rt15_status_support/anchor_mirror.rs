use super::*;

#[test]
fn runtime_15_status_support_anchor_mirror_is_child_owned() {
    let parent = read_runtime_src(STATUS_SUPPORT_ROWS_PATH);
    let anchor_mirror = read_runtime_src(STATUS_SUPPORT_ANCHOR_MIRROR_PATH);
    let status_rows = read_runtime_src(STATUS_SUPPORT_ROW_DATA_ANCHOR_MIRROR_ROW_PATH);
    let status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/row_data_maps/root_runtime_maps.rs",
    );
    let date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/row_data_maps/root_runtime_maps.rs",
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let framework_plan = read_repo(
        "docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md",
    );
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    assert_contains_all(
        "status-support parent routes anchor mirror child",
        &parent,
        &[
            "#[path = \"status_support/anchor_mirror.rs\"]",
            "mod anchor_mirror;",
            "anchor_mirror::STATUS_SUPPORT_ROW_DATA_ANCHOR_MIRROR",
        ],
    );
    for forbidden in [
        "pub(super) const STATUS_SUPPORT_ROW_DATA_ANCHOR_MIRROR: &str = r#",
        "Runtime 15 M3 structure-support expected-slice guard body child split",
        "runtime_15_structure_support_expected_slice_guard_body_child_split_static_passed_cargo_deferred",
        "Runtime 15 M3 render shader template assembly assertion contract child-owner split",
    ] {
        assert!(
            !parent.contains(forbidden),
            "status_support.rs should delegate anchor mirror body to child owner; found {forbidden}"
        );
    }
    assert_contains_all(
        "status-support anchor mirror child owns historical anchors",
        &anchor_mirror,
        &[
            "pub(super) const STATUS_SUPPORT_ROW_DATA_ANCHOR_MIRROR: &str = r#",
            "Runtime 15 M3 structure-support expected-slice guard body child split",
            "runtime_15_status_support_expected_slice_map_child_split_static_passed_cargo_blocked_render_environment_exports",
            "Runtime 15 M3 render shader template assembly assertion contract child-owner split",
            "runtime_15_shader_prewarm_manifest_guard_child_owner_split_static_passed_cargo_deferred",
        ],
    );

    let status_anchors = [
        ANCHOR_MIRROR_CHILD_SPLIT_STATUS_NAME,
        ANCHOR_MIRROR_CHILD_SPLIT_STATUS_ID,
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/anchor_mirror.rs",
        "structure_convention/test_file_budget/row_data/rt15_status_support/anchor_mirror.rs",
        ANCHOR_MIRROR_CHILD_SPLIT_GUARD_NAME,
        "Cargo gate deferred",
    ];
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("Frameworks 02 plan", framework_plan.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("status-support runtime row-data rows", status_rows.as_str()),
    ] {
        assert_contains_all(label, source, &status_anchors);
    }
    assert_contains_all(
        "M3 status-support status map records anchor mirror child split",
        &status_map,
        &[
            ANCHOR_MIRROR_CHILD_SPLIT_STATUS_NAME,
            ANCHOR_MIRROR_CHILD_SPLIT_STATUS_ID,
        ],
    );
    assert_contains_all(
        "M3 status-support date map records anchor mirror child split",
        &date_map,
        &[ANCHOR_MIRROR_CHILD_SPLIT_STATUS_NAME, "2026-07-06"],
    );
}
