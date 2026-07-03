use super::super::assert_contains_all;
use super::{read_repo, read_runtime_src, DEAD_CODE_ALLOW_ATTRIBUTE};

#[test]
fn runtime_15_runtime_owned_dead_code_suppression_cleanup() {
    let asset_worker_pool = read_runtime_src("asset/pipeline/worker_pool.rs");
    let asset_worker_pool_tests = read_runtime_src("asset/tests/pipeline/worker_pool.rs");
    let module_entry = read_runtime_src("core/runtime/state/module_entry.rs");
    let runtime_devtools = read_runtime_src("core/runtime/diagnostics/devtools.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    for (label, source) in [
        ("asset worker pool", asset_worker_pool.as_str()),
        ("core runtime module entry", module_entry.as_str()),
    ] {
        assert!(
            !source.contains(DEAD_CODE_ALLOW_ATTRIBUTE),
            "{label} should expose live code or test-only reads instead of dead-code suppression"
        );
    }

    assert_contains_all(
        "asset worker pool test-only receiver guard",
        &asset_worker_pool,
        &[
            "request_rx_guard: Option<ChannelReceiver<AssetRequest>>",
            "pub(crate) fn request_channel_guard_is_alive_for_test(&self) -> bool",
            "self.request_rx_guard.is_some()",
        ],
    );
    assert_contains_all(
        "asset worker pool tests read the guard",
        &asset_worker_pool_tests,
        &["pool.request_channel_guard_is_alive_for_test()"],
    );
    assert_contains_all(
        "module entry descriptor is a live diagnostics source",
        &module_entry,
        &[
            "pub(crate) descriptor: ModuleDescriptor",
            "pub(crate) fn descriptor(&self) -> &ModuleDescriptor",
            "&self.descriptor",
        ],
    );
    assert_contains_all(
        "runtime devtools consumes the module descriptor accessor",
        &runtime_devtools,
        &[
            "let descriptor = entry.descriptor();",
            "name: descriptor.name.clone()",
            "driver_count: descriptor.drivers.len()",
        ],
    );

    for (label, source) in [
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
                "Runtime 15 F12 runtime-owned dead-code suppression cleanup",
                "runtime_15_runtime_owned_dead_code_suppression_cleanup_coremin_check_passed",
                "runtime_15_runtime_owned_dead_code_suppression_cleanup",
            ],
        );
    }
}
