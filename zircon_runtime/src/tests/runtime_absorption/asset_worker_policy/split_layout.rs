const PARENT_SOURCE: &str = include_str!("../asset_worker_policy.rs");
const WORKER_POOL_SOURCE: &str = include_str!("worker_pool.rs");
const SPLIT_LAYOUT_SOURCE: &str = include_str!("split_layout.rs");

const RUNTIME_04_PLAN: &str =
    include_str!("../../../../../docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md");
const RUNTIME_11_PLAN: &str =
    include_str!("../../../../../docs/plans/zircon_runtime/runtime/11-job-system-task-model.md");
const RUNTIME_15_PLAN: &str = include_str!(
    "../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
);
const RUNTIME_INDEX: &str =
    include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
const STRUCTURE_CONVENTION_PLAN: &str =
    include_str!("../../../../../docs/plans/engine-code-structure-convention.md");
const REVIEW_FINDINGS_PLAN: &str =
    include_str!("../../../../../docs/plans/engine-code-review-findings-2026-06.md");
const MODULE_CONVENTION_DOC: &str =
    include_str!("../../../../../docs/zircon_runtime/structure/module-convention.md");
const FRAMEWORKS_02_PLAN: &str =
    include_str!("../../../../../docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md");
const SESSION_NOTE: &str = include_str!(
    "../../../../../.codex/sessions/20260612-0847-runtime-architecture-implementation.md"
);
const STATUS_ROW_DATA: &str = include_str!(
    "../plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards/runtime_structure_tests.rs"
);
const STATUS_MAP: &str = include_str!(
    "../plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/structure_route_maps.rs"
);
const DATE_MAP: &str = include_str!(
    "../plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/structure_route_maps.rs"
);

#[test]
fn runtime_15_asset_worker_policy_route_owner_is_folder_backed() {
    assert_contains_all(
        "asset_worker_policy route owner",
        PARENT_SOURCE,
        &[
            "#[path = \"asset_worker_policy/split_layout.rs\"]",
            "#[path = \"asset_worker_policy/worker_pool.rs\"]",
        ],
    );
    assert_parent_route_only();
    assert_child_owners_are_focused();
    assert_line_budget();
    assert_docs_and_status_mirror_split();
}

fn assert_parent_route_only() {
    assert!(
        !PARENT_SOURCE.contains("#[test]"),
        "asset_worker_policy.rs should route child owners instead of owning tests"
    );
    for forbidden in [
        "asset_worker_pool_matches_runtime_04_and_11_decisions",
        "AssetWorkerPoolOptions",
        "asset.worker.budgeted_threads",
    ] {
        assert!(
            !PARENT_SOURCE.contains(forbidden),
            "asset_worker_policy.rs should not retain `{forbidden}`"
        );
    }
}

fn assert_child_owners_are_focused() {
    assert_contains_all(
        "worker pool child",
        WORKER_POOL_SOURCE,
        &[
            "asset_worker_pool_matches_runtime_04_and_11_decisions",
            "AssetWorkerPoolOptions",
            "AssetWorkerPoolFrameSampler",
            "Runtime 11 M2.4",
            "worker_pool_frame_sampler_records_per_frame_completion_deltas",
        ],
    );
}

fn assert_line_budget() {
    for (label, source, max_lines) in [
        ("parent route owner", PARENT_SOURCE, 6),
        ("worker pool child", WORKER_POOL_SOURCE, 140),
        ("split layout guard", SPLIT_LAYOUT_SOURCE, 180),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count <= max_lines,
            "{label} has {line_count} lines; expected at most {max_lines}"
        );
    }
}

fn assert_docs_and_status_mirror_split() {
    for (label, source) in [
        ("Runtime 04 plan", RUNTIME_04_PLAN),
        ("Runtime 11 plan", RUNTIME_11_PLAN),
        ("Runtime 15 plan", RUNTIME_15_PLAN),
        ("runtime index", RUNTIME_INDEX),
        ("structure convention plan", STRUCTURE_CONVENTION_PLAN),
        ("review findings plan", REVIEW_FINDINGS_PLAN),
        ("module convention doc", MODULE_CONVENTION_DOC),
        ("Frameworks 02 plan", FRAMEWORKS_02_PLAN),
        ("session note", SESSION_NOTE),
        ("status row data", STATUS_ROW_DATA),
        ("status map", STATUS_MAP),
    ] {
        assert!(
            source.contains(
                "runtime_15_asset_worker_policy_route_owner_split_static_passed_cargo_deferred"
            ),
            "{label} should mirror the asset_worker_policy route-owner split status"
        );
    }
    assert!(
        DATE_MAP.contains("Runtime 15 M3 asset-worker-policy route-owner split"),
        "date map should mirror the asset_worker_policy route-owner split slice"
    );
    assert_contains_all(
        "module convention doc",
        MODULE_CONVENTION_DOC,
        &[
            "asset_worker_policy/worker_pool.rs",
            "asset_worker_policy/split_layout.rs",
            "runtime_15_asset_worker_policy_route_owner_is_folder_backed",
        ],
    );
    assert_contains_all(
        "Frameworks 02 plan",
        FRAMEWORKS_02_PLAN,
        &[
            "frameworks_02_m3_asset_worker_policy_route_owner_split_static_passed_cargo_deferred",
            "Runtime 15 M3 asset-worker-policy route-owner split",
        ],
    );
}

fn assert_contains_all(label: &str, source: &str, required: &[&str]) {
    for anchor in required {
        assert!(
            source.contains(anchor),
            "{label} should contain split anchor `{anchor}`"
        );
    }
}
