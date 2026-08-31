use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_neutral_rhi_device_errors_are_child_owned() {
    let parent = read_repo("zircon_runtime/crates/zr_rhi/src/device.rs");
    let error = read_repo("zircon_runtime/crates/zr_rhi/src/device/error.rs");

    assert_contains_all(
        "neutral RHI device delegates the public error contract",
        &parent,
        &["mod error;", "pub use self::error::RhiError;"],
    );
    assert_contains_all(
        "neutral RHI error child owns typed device and submission failures",
        &error,
        &[
            "pub enum RhiError",
            "SubmissionSequenceExhausted",
            "SubmissionPollSequenceExhausted",
            "NativeDevicePoll",
            "impl From<UnsupportedRenderOperation> for RhiError",
        ],
    );
    assert!(
        !parent.contains("pub enum RhiError"),
        "zr_rhi/device.rs should delegate the error contract to device/error.rs"
    );
    for (path, source) in [
        ("zr_rhi/device.rs", parent.as_str()),
        ("zr_rhi/device/error.rs", error.as_str()),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 production-file soft budget; got {line_count} lines"
        );
    }
}

#[test]
fn runtime_15_rhi_wgpu_device_command_list_is_child_owner() {
    let parent = read_repo("zircon_runtime/crates/zr_rhi_wgpu/src/device.rs");
    let command_list = read_repo("zircon_runtime/crates/zr_rhi_wgpu/src/device/command_list.rs");
    let construction = read_repo("zircon_runtime/crates/zr_rhi_wgpu/src/device/construction.rs");
    let contract_caps = read_repo("zircon_runtime/crates/zr_rhi_wgpu/src/device/contract_caps.rs");
    let rhi_wgpu_root = read_repo("zircon_runtime/crates/zr_rhi_wgpu/src/lib.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let rhi_doc = read_repo("docs/zircon_runtime/rhi/descriptors.md");

    assert_contains_all(
        "deterministic RHI contract device delegates command-list recording and keeps test state",
        &parent,
        &[
            "mod command_list;",
            "mod construction;",
            "pub(crate) use self::command_list::DeterministicRhiContractCommandList;",
            "pub(crate) struct DeterministicRhiContractDevice",
            "pub(super) struct DeterministicRhiContractDeviceState",
            "impl RenderDevice for DeterministicRhiContractDevice",
            "fn create_command_list(",
            "DeterministicRhiContractCommandList::new(",
            "fn submit(&self, command_list: Box<dyn CommandList>)",
            "fn lock_state(&self) -> MutexGuard<'_, DeterministicRhiContractDeviceState>",
        ],
    );
    for moved_owner in [
        "pub(crate) struct DeterministicRhiContractCommandList",
        "impl CommandList for DeterministicRhiContractCommandList",
        "fn push_debug_marker(&mut self, label: &str)",
        "fn copy_buffer_to_texture(",
        "fn begin_render_pass(",
        "fn set_vertex_buffer(",
        "fn draw_indexed(",
        "fn dispatch_compute(&mut self, x: u32, y: u32, z: u32)",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "rhi_wgpu/device.rs should delegate {moved_owner} to rhi_wgpu/device/command_list.rs"
        );
    }
    assert_contains_all(
        "deterministic RHI contract command-list child owns command recording",
        &command_list,
        &[
            "pub(crate) struct DeterministicRhiContractCommandList",
            "impl DeterministicRhiContractCommandList",
            "impl CommandList for DeterministicRhiContractCommandList",
            "CommandListCommand::DebugMarker",
            "CommandListCommand::CopyBufferToTexture",
            "CommandListCommand::BeginRenderPass",
            "CommandListCommand::SetVertexBuffer",
            "CommandListCommand::DrawIndexed",
            "CommandListCommand::DispatchCompute",
        ],
    );
    assert_contains_all(
        "deterministic RHI construction child owns device initialization policy",
        &construction,
        &[
            "impl DeterministicRhiContractDevice",
            "pub(crate) fn new_headless()",
            "fn new_headless_with_identity_and_config(",
            "deterministic_contract_caps()",
            "SubmissionHistory::new(submission_limits)",
        ],
    );
    assert_contains_all(
        "deterministic RHI capability child owns the backend receipt",
        &contract_caps,
        &["RenderBackendCaps::new(\"deterministic-rhi-contract-test\")"],
    );
    assert_contains_all(
        "RHI WGPU root keeps the deterministic contract device test-only",
        &rhi_wgpu_root,
        &["use device::{DeterministicRhiContractCommandList, DeterministicRhiContractDevice};"],
    );
    assert!(
        !rhi_wgpu_root.contains("as WgpuCommandList")
            && !rhi_wgpu_root.contains("as WgpuRenderDevice"),
        "the deterministic contract test types must not retain production-shaped WGPU aliases"
    );
    assert!(
        !parent.contains("WgpuRenderDeviceState"),
        "the deterministic contract device must not retain the production-shaped state alias"
    );
    assert!(
        rhi_wgpu_root
            .split_whitespace()
            .collect::<String>()
            .contains("#[cfg(test)]moddevice;"),
        "the deterministic RHI contract device module must remain cfg(test)-only"
    );

    for (path, source) in [
        ("rhi_wgpu/device.rs", parent.as_str()),
        ("rhi_wgpu/device/command_list.rs", command_list.as_str()),
        ("rhi_wgpu/device/construction.rs", construction.as_str()),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 production-file soft budget; got {line_count} lines"
        );
    }
}

#[test]
fn runtime_15_rhi_wgpu_production_device_capabilities_are_child_owned() {
    let parent = read_repo("zircon_runtime/crates/zr_rhi_wgpu/src/production/device.rs");
    let capabilities =
        read_repo("zircon_runtime/crates/zr_rhi_wgpu/src/production/device/capabilities.rs");
    let tests = read_repo("zircon_runtime/crates/zr_rhi_wgpu/src/production/tests.rs");
    let capability_tests =
        read_repo("zircon_runtime/crates/zr_rhi_wgpu/src/production/tests/capabilities.rs");

    assert_contains_all(
        "production RHI device delegates capability receipt construction",
        &parent,
        &[
            "pub(super) mod capabilities;",
            "use capabilities::production_caps;",
            "let caps = production_caps(&adapter, &device, &profile);",
        ],
    );
    assert_contains_all(
        "production RHI capability child owns native-to-neutral mapping",
        &capabilities,
        &[
            "fn production_caps_from_wgpu(",
            "RenderOperation::MultiDrawIndirectCount",
            "with_pipeline_statistics_query",
            "fn adapter_class_label(",
        ],
    );
    for moved_owner in [
        "fn production_caps_from_wgpu(",
        "fn indirect_operation_support(",
        "fn adapter_class_label(",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "production/device.rs should delegate {moved_owner} to production/device/capabilities.rs"
        );
    }
    assert_contains_all(
        "production RHI tests delegate capability behavior to a folder-backed owner",
        &tests,
        &["#[path = \"tests/capabilities.rs\"]", "mod capabilities;"],
    );
    assert!(
        !tests.contains("fn production_capability_view_advertises_only_implemented_neutral_abi("),
        "production/tests.rs should delegate capability behavior to production/tests/capabilities.rs"
    );
    for (path, source) in [
        ("production/device.rs", parent.as_str()),
        ("production/device/capabilities.rs", capabilities.as_str()),
        ("production/tests.rs", tests.as_str()),
        (
            "production/tests/capabilities.rs",
            capability_tests.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 production-file soft budget; got {line_count} lines"
        );
    }
}
