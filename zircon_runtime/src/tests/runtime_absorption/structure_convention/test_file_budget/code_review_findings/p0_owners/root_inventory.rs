use super::*;

#[test]
fn runtime_15_p0_robustness_root_inventory_is_child_owned() {
    let parent = read_runtime_src(STRUCTURE_GUARD_OWNER);
    let status_rows = p0_robustness_status_row_source();
    let status_map = format!(
        "{}\n{}",
        read_runtime_src(REVIEW_GUARD_STATUS_MAP),
        read_runtime_src(REVIEW_GUARD_P0_STATUS_MAP)
    );
    let date_map = format!(
        "{}\n{}",
        read_runtime_src(REVIEW_GUARD_DATE_MAP),
        read_runtime_src(REVIEW_GUARD_P0_DATE_MAP)
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    for (module_name, child_path, anchor) in P0_ROOT_CHILDREN {
        let path_attr = format!("#[path = \"p0_owners/{module_name}.rs\"]");
        assert_contains_all(
            "P0 robustness parent mounts root-inventory child",
            &parent,
            &[path_attr.as_str(), *module_name],
        );

        let child_source = read_runtime_src(child_path);
        assert_contains_all(child_path, &child_source, &[*anchor]);
        assert!(
            child_source.lines().count() < P0_ROOT_CHILD_LINE_BUDGET,
            "{child_path} should stay below its root inventory child budget"
        );
    }

    for forbidden in [
        "pub(super) struct P0RobustnessSources",
        "pub(super) const STRUCTURE_GUARD_OWNER:",
        "pub(super) const REVIEW_GUARDS:",
        "pub(super) const FOLDER_BACKED_CHILDREN:",
        "pub(super) fn read_p0_robustness_sources",
        "pub(super) fn folder_backed_child_sources",
    ] {
        assert!(
            !parent.contains(forbidden),
            "P0 robustness parent should delegate root inventory anchor `{forbidden}`"
        );
    }

    let status_anchors = [
        P0_ROOT_INVENTORY_SLICE,
        P0_ROOT_INVENTORY_STATUS,
        STRUCTURE_GUARD_OWNER,
        P0_ROOT_PATHS_CHILD,
        P0_ROOT_STATUSES_CHILD,
        P0_ROOT_CHILD_ROWS_CHILD,
        P0_ROOT_SOURCES_CHILD,
        P0_ROOT_INVENTORY_CHILD,
        P0_ROOT_INVENTORY_GUARD,
        "Cargo gate deferred",
    ];
    for (label, source) in [
        ("P0 robustness row data", status_rows.as_str()),
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
    ] {
        assert_contains_all(label, source, &status_anchors);
    }
    assert_contains_all(
        "M3 review status map records P0 robustness root inventory split",
        &status_map,
        &[P0_ROOT_INVENTORY_SLICE, P0_ROOT_INVENTORY_STATUS],
    );
    assert_contains_all(
        "M3 review date map records P0 robustness root inventory split",
        &date_map,
        &[P0_ROOT_INVENTORY_SLICE, P0_ROOT_INVENTORY_DATE],
    );
}
