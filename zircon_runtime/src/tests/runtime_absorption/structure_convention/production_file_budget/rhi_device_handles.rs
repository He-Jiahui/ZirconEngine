use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_rhi_device_handles_are_child_owner() {
    let parent = read_runtime_src("rhi/device.rs");
    let handles = read_runtime_src("rhi/device/handles.rs");
    let rhi_root = read_runtime_src("rhi/mod.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let rhi_doc = read_repo("docs/zircon_runtime/rhi/descriptors.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m4.rs",
    );
    let status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15.rs",
    );
    let date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15.rs",
    );

    assert_contains_all(
        "RHI device parent keeps device contracts and re-exports handle owner",
        &parent,
        &[
            "mod handles;",
            "pub use self::handles::{",
            "BindGroupHandle",
            "BufferHandle",
            "TextureHandle",
            "pub enum RhiError",
            "pub enum CommandListCommand",
            "pub trait CommandList",
            "pub trait RenderDevice",
        ],
    );
    for moved_owner in [
        "pub struct BufferHandle(u64);",
        "pub struct TextureHandle(u64);",
        "pub struct SamplerHandle(u64);",
        "pub struct BindGroupLayoutHandle(u64);",
        "pub struct BindGroupHandle(u64);",
        "pub struct ShaderModuleHandle(u64);",
        "pub struct PipelineLayoutHandle(u64);",
        "pub struct PipelineHandle(u64);",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "rhi/device.rs should delegate {moved_owner} to rhi/device/handles.rs"
        );
    }
    assert_contains_all(
        "RHI device handle child owns all neutral resource handle newtypes",
        &handles,
        &[
            "pub struct BufferHandle(u64);",
            "impl BufferHandle",
            "pub struct TextureHandle(u64);",
            "impl TextureHandle",
            "pub struct SamplerHandle(u64);",
            "pub struct BindGroupLayoutHandle(u64);",
            "pub struct BindGroupHandle(u64);",
            "pub struct ShaderModuleHandle(u64);",
            "pub struct PipelineLayoutHandle(u64);",
            "pub struct PipelineHandle(u64);",
        ],
    );
    assert_contains_all(
        "RHI root still exports handle names through rhi::device",
        &rhi_root,
        &[
            "pub use device::{",
            "BufferHandle",
            "TextureHandle",
            "SamplerHandle",
            "BindGroupLayoutHandle",
            "BindGroupHandle",
            "ShaderModuleHandle",
            "PipelineLayoutHandle",
            "PipelineHandle",
        ],
    );

    for (path, source) in [
        ("rhi/device.rs", parent.as_str()),
        ("rhi/device/handles.rs", handles.as_str()),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 production-file soft budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("RHI descriptor doc", rhi_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M4 RHI device handle owner split",
                "runtime_15_rhi_device_handles_owner_split_static_passed_cargo_deferred",
                "rhi/device.rs",
                "rhi/device/handles.rs",
                "runtime_15_rhi_device_handles_are_child_owner",
            ],
        );
    }
    assert_contains_all(
        "status-output status map",
        &status_map,
        &[
            "Runtime 15 M4 RHI device handle owner split",
            "runtime_15_rhi_device_handles_owner_split_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "status-output date map",
        &date_map,
        &["Runtime 15 M4 RHI device handle owner split", "2026-06-24"],
    );
}
