use thiserror::Error;

use super::types::{RenderGraphExternalResourceType, RenderGraphResourceKind};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RenderGraphError {
    #[error("render graph pass `{pass}` is unknown")]
    UnknownPass { pass: usize },
    #[error(
        "render graph pass handle `{pass}` belongs to builder generation {handle_generation}, expected {builder_generation}"
    )]
    ForeignPass {
        pass: usize,
        handle_generation: u64,
        builder_generation: u64,
    },
    #[error("render graph resource `{resource}` is unknown")]
    UnknownResource { resource: String },
    #[error("render graph resource `{resource}` has no declaration during compilation")]
    ResourceDeclarationMissing { resource: String },
    #[error("render graph resource `{resource}` exhausted its logical write versions")]
    ResourceVersionExhausted { resource: String },
    #[error(
        "render graph {kind:?} handle `{index}` belongs to builder generation {handle_generation}, expected {builder_generation}"
    )]
    ForeignResource {
        kind: RenderGraphResourceKind,
        index: usize,
        handle_generation: u64,
        builder_generation: u64,
    },
    #[error("render graph resource name `{resource}` is declared more than once")]
    DuplicateResourceName { resource: String },
    #[error(
        "render graph external alias group `{alias_group}` mixes {expected:?} and {found:?} resources"
    )]
    ExternalAliasResourceTypeMismatch {
        alias_group: String,
        expected: RenderGraphExternalResourceType,
        found: RenderGraphExternalResourceType,
    },
    #[error(
        "render graph compute pass `{pass}` dispatch resource `{resource}` must declare {required_access}"
    )]
    ComputeDispatchResourceNotDeclared {
        pass: String,
        resource: String,
        required_access: &'static str,
    },
    #[error(
        "render graph compute pass `{pass}` binding `{binding}` resource `{resource}` must declare {required_access}"
    )]
    ComputeBindingResourceNotDeclared {
        pass: String,
        binding: u32,
        resource: String,
        required_access: &'static str,
    },
    #[error("render graph compute pass `{pass}` has execution metadata but no workload")]
    ComputePassMetadataMissingWorkload { pass: String },
    #[error("render graph compute pass `{pass}` has an empty entry point")]
    ComputePassEntryPointEmpty { pass: String },
    #[error("render graph compute pass `{pass}` has an empty WGSL shader source")]
    ComputePassShaderSourceEmpty { pass: String },
    #[error("render graph compute pass `{pass}` has an invalid zero workgroup dimension")]
    InvalidComputeWorkgroupSize { pass: String },
    #[error("render graph compute pass `{pass}` declares binding `{binding}` more than once")]
    DuplicateComputeBinding { pass: String, binding: u32 },
    #[error(
        "render graph compute pass `{pass}` binding `{binding}` selects buffer offset {offset} on a non-buffer binding"
    )]
    ComputeBufferOffsetBindingNotBuffer {
        pass: String,
        binding: u32,
        offset: u64,
    },
    #[error(
        "render graph compute pass `{pass}` binding `{binding}` selects texture mip {mip_level} on a non-texture binding"
    )]
    ComputeTextureMipBindingNotTexture {
        pass: String,
        binding: u32,
        mip_level: u32,
    },
    #[error(
        "render graph compute pass `{pass}` binding `{binding}` selects texture mip {mip_level} for external texture `{resource}`, but mip views require a transient texture"
    )]
    ComputeTextureMipRequiresTransientTexture {
        pass: String,
        binding: u32,
        resource: String,
        mip_level: u32,
    },
    #[error(
        "render graph compute pass `{pass}` binding `{binding}` selects mip {mip_level} for texture `{resource}`, but it has {mip_levels} mip levels"
    )]
    ComputeTextureMipOutOfRange {
        pass: String,
        binding: u32,
        resource: String,
        mip_level: u32,
        mip_levels: u32,
    },
    #[error(
        "render graph compute pass `{pass}` per-pixel local size {local_size:?} does not match workgroup size {workgroup_size:?}"
    )]
    PerPixelComputeWorkgroupMismatch {
        pass: String,
        local_size: [u32; 2],
        workgroup_size: [u32; 3],
    },
    #[error(
        "render graph compute pass `{pass}` indirect dispatch offset {offset} must be aligned to {alignment} bytes"
    )]
    ComputeIndirectDispatchOffsetUnaligned {
        pass: String,
        offset: u64,
        alignment: u64,
    },
    #[error("render graph `{graph_name}` contains a dependency cycle")]
    CycleDetected { graph_name: String },
    #[error(
        "render graph pass `{pass}` reads resource `{resource}` before any producer writes it"
    )]
    ReadBeforeProducer { resource: String, pass: String },
    #[error(
        "render graph pass `{pass}` loads transient attachment `{resource}` before any producer writes it"
    )]
    LoadBeforeProducer { resource: String, pass: String },
    #[error(
        "render graph pass `{pass}` reads transient attachment `{resource}` after producer `{producer}` discarded it"
    )]
    ReadAfterDiscardedStore {
        resource: String,
        pass: String,
        producer: String,
    },
    #[error("render graph `{graph_name}` has no present, readback, persistent, or side-effect cull root")]
    MissingCullRoot { graph_name: String },
}
