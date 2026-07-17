use super::super::super::super::super::super::*;
use super::super::*;
use super::*;

#[test]
fn runtime_15_typed_error_native_plugin_loader_routes_child_split_status_is_current() {
    let status_rows = review_guard_status_rows_source();
    let status_map = review_guard_status_map_source();
    let date_map = review_guard_date_map_source();
    let runtime_15_plan =
        read_repo(
            "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md",
        );
    let runtime_index = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
    );
    let review_findings = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
    );
    let structure_convention = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
    );
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    for (label, source) in [
        ("typed-error structure row data", status_rows.as_str()),
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
                TYPED_ERROR_NATIVE_STRUCTURE_ROUTES_SPLIT,
                TYPED_ERROR_NATIVE_STRUCTURE_ROUTES_STATUS,
                TYPED_ERROR_NATIVE_STRUCTURE_ROUTES_CHILD,
                TYPED_ERROR_NATIVE_STRUCTURE_ROUTES_CHILD_OWNERSHIP_CHILD,
                TYPED_ERROR_NATIVE_STRUCTURE_ROUTES_TOP_LEVEL_CHILD,
                TYPED_ERROR_NATIVE_STRUCTURE_ROUTES_ABI_SURFACES_CHILD,
                TYPED_ERROR_NATIVE_STRUCTURE_ROUTES_PLUGIN_DESCRIPTOR_CHILD,
                TYPED_ERROR_NATIVE_STRUCTURE_ROUTES_LIVE_HOST_CHILD,
                TYPED_ERROR_NATIVE_STRUCTURE_ROUTES_LIFECYCLE_PATHS_CHILD,
                TYPED_ERROR_NATIVE_STRUCTURE_ROUTES_REPLAY_RUNTIME_CHILD,
                TYPED_ERROR_NATIVE_STRUCTURE_ROUTES_MANIFEST_SOURCES_CHILD,
                TYPED_ERROR_NATIVE_STRUCTURE_ROUTES_STATUS_CHILD,
                TYPED_ERROR_NATIVE_STRUCTURE_ROUTES_OWNERSHIP_GUARD,
                TYPED_ERROR_NATIVE_STRUCTURE_ROUTES_STATUS_GUARD,
                "Cargo gate deferred",
            ],
        );
    }
    assert_contains_all(
        "M3 review status map records typed-error native plugin loader routes split",
        &status_map,
        &[
            TYPED_ERROR_NATIVE_STRUCTURE_ROUTES_SPLIT,
            TYPED_ERROR_NATIVE_STRUCTURE_ROUTES_STATUS,
        ],
    );
    assert_contains_all(
        "M3 review date map records typed-error native plugin loader routes split",
        &date_map,
        &[
            TYPED_ERROR_NATIVE_STRUCTURE_ROUTES_SPLIT,
            TYPED_ERROR_NATIVE_STRUCTURE_ROUTES_DATE,
        ],
    );
}
