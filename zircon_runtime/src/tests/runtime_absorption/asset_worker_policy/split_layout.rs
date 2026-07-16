const PARENT_SOURCE: &str = include_str!("../asset_worker_policy.rs");
const WORKER_POOL_SOURCE: &str = include_str!("worker_pool.rs");
const SPLIT_LAYOUT_SOURCE: &str = include_str!("split_layout.rs");

const RUNTIME_15_OUTPUT_RECORDS: &str = include_str!(
    "../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md"
);
const MODULE_CONVENTION_DOC: &str =
    include_str!("../../../../../docs/zircon_runtime/structure/module-convention.md");
const FRAMEWORKS_02_OUTPUT_RECORDS: &str = include_str!(
    "../../../../../docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md"
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
    assert!(
        RUNTIME_15_OUTPUT_RECORDS.contains(
            "runtime_15_asset_worker_policy_route_owner_split_static_passed_cargo_deferred"
        ),
        "Runtime 15 output records should own the asset_worker_policy route-owner split status"
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
        "Frameworks 02 output records",
        FRAMEWORKS_02_OUTPUT_RECORDS,
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
