use super::super::*;
use super::*;

pub(super) fn assert_f8_route_ownership_status_mirrors_are_current() {
    let status_rows = f8_status_row_source();
    let status_map = read_runtime_src(REVIEW_GUARD_STATUS_MAP);
    let date_map = read_runtime_src(REVIEW_GUARD_DATE_MAP);
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");

    for (label, source) in [
        ("F8 row data", status_rows.as_str()),
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("runtime architecture session note", session_note.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                F8_ROUTE_OWNERSHIP_CHILD_SPLIT_SLICE,
                F8_ROUTE_OWNERSHIP_CHILD_SPLIT_STATUS,
                F8_ROUTE_PARENT_ROUTES_CHILD,
                F8_ROUTE_DESCRIPTOR_BUILDER_ROUTES_CHILD,
                F8_ROUTE_DESCRIPTOR_PRIVACY_ROUTES_CHILD,
                F8_ROUTE_LEAF_OWNERS_CHILD,
                F8_ROUTE_CHILD_OWNERSHIP_CHILD,
                F8_ROUTE_STATUS_MIRRORS_CHILD,
                F8_ROUTE_OWNERSHIP_CHILD_SPLIT_GUARD,
                F8_ROUTE_STATUS_MIRROR_GUARD,
                "Cargo gate deferred",
            ],
        );
    }
    assert_contains_all(
        "M3 review status map records F8 route ownership child split",
        &status_map,
        &[
            F8_ROUTE_OWNERSHIP_CHILD_SPLIT_SLICE,
            F8_ROUTE_OWNERSHIP_CHILD_SPLIT_STATUS,
        ],
    );
    assert_contains_all(
        "M3 review date map records F8 route ownership child split",
        &date_map,
        &[
            F8_ROUTE_OWNERSHIP_CHILD_SPLIT_SLICE,
            F8_ROUTE_OWNERSHIP_CHILD_SPLIT_DATE,
        ],
    );
}
