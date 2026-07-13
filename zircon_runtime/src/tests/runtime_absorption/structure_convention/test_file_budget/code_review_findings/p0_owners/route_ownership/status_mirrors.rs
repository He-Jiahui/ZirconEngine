use super::super::*;
use super::*;

pub(super) fn assert_p0_route_ownership_status_mirrors_are_current() {
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

    for (label, source) in [
        ("P0 row data", status_rows.as_str()),
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                P0_ROUTE_OWNERSHIP_CHILD_SPLIT_SLICE,
                P0_ROUTE_OWNERSHIP_CHILD_SPLIT_STATUS,
                P0_ROUTE_OWNERSHIP_CHILD,
                P0_ROUTE_PARENT_ROUTES_CHILD,
                P0_ROUTE_LEAF_OWNERS_CHILD,
                P0_ROUTE_CHILD_OWNERSHIP_CHILD,
                P0_ROUTE_STATUS_MIRRORS_CHILD,
                GUARD,
                P0_ROUTE_OWNERSHIP_CHILD_SPLIT_GUARD,
                P0_ROUTE_STATUS_MIRROR_GUARD,
                "Cargo gate deferred",
            ],
        );
    }
    assert_contains_all(
        "M3 review status map records P0 route ownership child split",
        &status_map,
        &[
            P0_ROUTE_OWNERSHIP_CHILD_SPLIT_SLICE,
            P0_ROUTE_OWNERSHIP_CHILD_SPLIT_STATUS,
        ],
    );
    assert_contains_all(
        "M3 review date map records P0 route ownership child split",
        &date_map,
        &[
            P0_ROUTE_OWNERSHIP_CHILD_SPLIT_SLICE,
            P0_ROUTE_OWNERSHIP_CHILD_SPLIT_DATE,
        ],
    );
}
