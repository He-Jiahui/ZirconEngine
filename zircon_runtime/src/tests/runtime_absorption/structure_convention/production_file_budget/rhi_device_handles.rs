use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_rhi_device_handles_are_child_owner() {
    let parent = read_repo("zircon_runtime/crates/zr_rhi/src/device.rs");
    let handles = read_repo("zircon_runtime/crates/zr_rhi/src/device/handles.rs");
    let rhi_root = read_repo("zircon_runtime/crates/zr_rhi/src/lib.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let rhi_doc = read_repo("docs/zircon_runtime/rhi/descriptors.md");

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
}
