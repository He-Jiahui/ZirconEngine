use super::{assert_contains_all, repo_path, runtime_src_path};

const LOCK_UNWRAP_CALL: &str = concat!(".lock().", "unwrap()");

#[test]
fn runtime_15_rhi_wgpu_render_device_lock_poison_recovery_guard_covers_device_state() {
    let wgpu_device = read_repo("zircon_runtime/crates/zr_rhi_wgpu/src/device.rs");
    let structure_parent = read_runtime_src("tests/runtime_absorption/structure_convention.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let rhi_doc = read_repo("docs/zircon_runtime/rhi/descriptors.md");

    assert_contains_all(
        "RHI WGPU render device poison recovery",
        &wgpu_device,
        &[
            "use std::sync::{Arc, Mutex, MutexGuard};",
            "fn lock_state(&self) -> MutexGuard<'_, DeterministicRhiContractDeviceState>",
            ".unwrap_or_else(|poisoned| poisoned.into_inner())",
            "self.lock_state().transient_allocator_stats()",
            "wgpu_render_device_state_accessors_recover_poisoned_lock",
        ],
    );
    assert_contains_all(
        "RHI WGPU lock poison guard mount",
        &structure_parent,
        &[
            "#[path = \"structure_convention/rhi_wgpu_lock_poison.rs\"]",
            "mod rhi_wgpu_lock_poison;",
        ],
    );
    assert_no_direct_lock_unwrap_in_production("RHI WGPU render device", &wgpu_device);
}

fn assert_no_direct_lock_unwrap_in_production(label: &str, source: &str) {
    let production = production_section(source);
    assert!(
        !production.contains(LOCK_UNWRAP_CALL),
        "{label} production code should use poison-safe lock helpers instead of {LOCK_UNWRAP_CALL}"
    );
}

fn production_section(source: &str) -> &str {
    source.split("\n#[cfg(test)]").next().unwrap_or(source)
}

fn read_runtime_src(relative: &str) -> String {
    std::fs::read_to_string(runtime_src_path(relative))
        .unwrap_or_else(|error| panic!("failed to read runtime source `{relative}`: {error}"))
}

fn read_repo(relative: &str) -> String {
    std::fs::read_to_string(repo_path(relative))
        .unwrap_or_else(|error| panic!("failed to read repository file `{relative}`: {error}"))
}
