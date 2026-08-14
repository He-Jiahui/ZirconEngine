use super::super::assert_contains_all;
use super::{read_runtime_src, DEAD_CODE_ALLOW_ATTRIBUTE};

#[test]
fn runtime_15_runtime_owned_dead_code_suppression_cleanup() {
    let asset_worker_pool = read_runtime_src("asset/pipeline/worker_pool.rs");
    let asset_worker_pool_tests = read_runtime_src("asset/tests/pipeline/worker_pool.rs");
    let module_entry = read_runtime_src("core/runtime/state/module_entry.rs");
    let runtime_devtools = read_runtime_src("core/runtime/diagnostics/devtools.rs");

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
}
