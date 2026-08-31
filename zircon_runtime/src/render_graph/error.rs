use thiserror::Error;

use crate::rhi::{BufferUsage, TextureUsage};

use super::access::{
    RenderGraphResourceAccessIntent, RenderGraphResourceAccessMetadata, RenderGraphTextureAspect,
};
use super::types::{
    RenderGraphExternalResourceType, RenderGraphResourceAccessKind, RenderGraphResourceKind,
};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RenderGraphError {
    #[error(
        "render graph compute pass `{pass}` binding {binding} has no declared {access:?} access for resource `{resource}`"
    )]
    ComputeBindingAccessMissing {
        pass: String,
        binding: u32,
        resource: String,
        access: RenderGraphResourceAccessKind,
    },
    #[error(
        "render graph compute pass `{pass}` binding {binding} has {candidate_count} declared {access:?} accesses for resource `{resource}`"
    )]
    ComputeBindingAccessAmbiguous {
        pass: String,
        binding: u32,
        resource: String,
        access: RenderGraphResourceAccessKind,
        candidate_count: usize,
    },
    #[error(
        "render graph compute pass `{pass}` binding {binding} {access:?} access for resource `{resource}` expects {expected:?} but the compiled graph contains {actual:?}"
    )]
    ComputeBindingAccessScopeMismatch {
        pass: String,
        binding: u32,
        resource: String,
        access: RenderGraphResourceAccessKind,
        expected: RenderGraphResourceAccessMetadata,
        actual: RenderGraphResourceAccessMetadata,
    },
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
    #[error("render graph external access packet is invalid: {message}")]
    ExternalAccessPacketBuild { message: String },
    #[error("render graph access scope tracker state is inconsistent for identity {identity}")]
    AccessScopeTrackerStateMismatch { identity: usize },
    #[error("render graph resource `{resource}` exhausted its logical write versions")]
    ResourceVersionExhausted { resource: String },
    #[error(
        "render graph resource version from builder generation {handle_generation} cannot be used by builder generation {builder_generation}"
    )]
    ForeignResourceVersion {
        handle_generation: u64,
        builder_generation: u64,
    },
    #[error(
        "render graph pass `{pass}` consumes resource `{resource}` from producer `{producer}`, but the latest producer is `{latest_producer}`"
    )]
    ResourceVersionNotCurrent {
        pass: String,
        resource: String,
        producer: String,
        latest_producer: String,
    },
    #[error(
        "render graph pass `{pass}` consumes resource `{resource}` from producer `{producer}`, but that value is unavailable"
    )]
    ResourceVersionUnavailable {
        pass: String,
        resource: String,
        producer: String,
    },
    #[error(
        "render graph pass `{pass}` consumes resource `{expected_resource}` with a version token for `{producer_resource}`"
    )]
    ResourceVersionResourceMismatch {
        pass: String,
        expected_resource: String,
        producer_resource: String,
    },
    #[error(
        "render graph resource version producer pass `{producer_pass}` access `{producer_access}` is not a write"
    )]
    ResourceVersionProducerNotWrite {
        producer_pass: String,
        producer_access: usize,
    },
    #[error(
        "render graph resource version producer pass `{producer_pass}` access `{producer_access}` is missing"
    )]
    ResourceVersionProducerMissing {
        producer_pass: usize,
        producer_access: usize,
    },
    #[error(
        "render graph pass `{pass}` cannot consume its own produced resource version `{resource}`"
    )]
    ResourceVersionSelfDependency { pass: String, resource: String },
    #[error(
        "render graph pass `{pass}` uses a resource version token on `{resource}` without an attachment Load operation"
    )]
    ResourceVersionRequiresAttachmentLoad { pass: String, resource: String },
    #[error(
        "render graph pass `{pass}` consumes scope of resource `{resource}` that is not fully covered by producer `{producer}`"
    )]
    ResourceVersionScopeNotCovered {
        pass: String,
        resource: String,
        producer: String,
    },
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
        "render graph texture view alias `{alias}` selects mip range base {base_mip_level} count {mip_level_count:?} and array range base {base_array_layer} count {array_layer_count:?} outside parent `{parent_name}` with {mip_levels} mips and {array_layers} addressable layers"
    )]
    TextureViewAliasRangeOutOfBounds {
        alias: String,
        parent_name: String,
        base_mip_level: u32,
        mip_level_count: Option<u32>,
        mip_levels: u32,
        base_array_layer: u32,
        array_layer_count: Option<u32>,
        array_layers: u32,
    },
    #[error(
        "render graph texture view alias `{alias}` selects {aspect:?} aspect unsupported by parent `{parent_name}` format {format:?}"
    )]
    TextureViewAliasAspectUnsupported {
        alias: String,
        parent_name: String,
        aspect: RenderGraphTextureAspect,
        format: crate::rhi::TextureFormat,
    },
    #[error(
        "render graph texture view alias `{alias}` cannot use view alias `{parent_name}` as its parent; aliases must reference an allocated texture"
    )]
    TextureViewAliasParentIsAlias { alias: String, parent_name: String },
    #[error(
        "compiled render graph transient allocation {allocation_id} assigns resource `{resource}` an invalid inclusive pass interval [{first_pass}, {last_pass}]"
    )]
    TransientAllocationInvalidInterval {
        allocation_id: usize,
        resource: String,
        first_pass: usize,
        last_pass: usize,
    },
    #[error(
        "compiled render graph transient allocation {allocation_id} assigns overlapping inclusive intervals to `{first_resource}` [{first_start}, {first_end}] and `{second_resource}` [{second_start}, {second_end}]"
    )]
    TransientAllocationIntervalsOverlap {
        allocation_id: usize,
        first_resource: String,
        first_start: usize,
        first_end: usize,
        second_resource: String,
        second_start: usize,
        second_end: usize,
    },
    #[error(
        "render graph texture resource `{resource}` has a storage size that exceeds the supported u64 allocation range"
    )]
    TextureStorageSizeOverflow { resource: String },
    #[error(
        "compiled render graph transient {kind:?} reservations exceed the supported u64 allocation range"
    )]
    TransientAllocationBytesOverflow { kind: RenderGraphResourceKind },
    #[error(
        "compiled render graph total dense transient reservations exceed the supported u64 allocation range"
    )]
    TransientAllocationTotalBytesOverflow,
    #[error(
        "render graph texture resource `{resource}` requests SparseReserved residency, but no sparse residency provider is available"
    )]
    SparseTextureUnsupported { resource: String },
    #[error(
        "render graph texture resource `{resource}` requests STORAGE usage for format {format:?}, which is unsupported by the write-only storage texture ABI"
    )]
    TextureStorageUsageUnsupported {
        resource: String,
        format: crate::rhi::TextureFormat,
    },
    #[error(
        "render graph external texture resource `{resource}` declares a non-texture external binding"
    )]
    ExternalTextureBindingTypeMismatch { resource: String },
    #[error(
        "render graph external buffer resource `{resource}` declares a non-buffer external binding"
    )]
    ExternalBufferBindingTypeMismatch { resource: String },
    #[error(
        "render graph external buffer resource `{resource}` declares an invalid physical descriptor"
    )]
    ExternalBufferDescriptorInvalid { resource: String },
    #[error("render graph pass name `{pass}` is declared more than once")]
    DuplicatePassName { pass: String },
    #[error(
        "render graph pass `{pass}` has overlapping {access:?} scopes for resources `{first_resource}` (access {first_access}) and `{second_resource}` (access {second_access})"
    )]
    OverlappingPassResourceAccessScope {
        pass: String,
        first_resource: String,
        first_access: usize,
        second_resource: String,
        second_access: usize,
        access: RenderGraphResourceAccessKind,
    },
    #[error(
        "compiled render graph access table `{table}` has {actual} pass rows, expected {expected}"
    )]
    CompiledAccessTablePassCountMismatch {
        table: &'static str,
        expected: usize,
        actual: usize,
    },
    #[error(
        "compiled render graph access table `{table}` has {actual} rows for pass `{pass}`, expected {expected}"
    )]
    CompiledAccessTableAccessCountMismatch {
        table: &'static str,
        pass: String,
        expected: usize,
        actual: usize,
    },
    #[error(
        "compiled render graph pass `{pass}` access for resource `{resource}` declares kind {access_kind:?}, but the resource declaration is {declaration_kind:?}"
    )]
    CompiledAccessResourceKindMismatch {
        pass: String,
        resource: String,
        access_kind: RenderGraphResourceKind,
        declaration_kind: RenderGraphResourceKind,
    },
    #[error(
        "compiled render graph pass `{pass}` access {access} has no indexed versioned access key"
    )]
    CompiledAccessIndexEntryMissing { pass: String, access: usize },
    #[error(
        "render graph pass `{pass}` selects a texture range for resource `{resource}`, but no texture descriptor is available"
    )]
    TextureAccessRangeRequiresTexture { pass: String, resource: String },
    #[error(
        "render graph pass `{pass}` selects a buffer range for resource `{resource}`, but no buffer descriptor is available"
    )]
    BufferAccessRangeRequiresBuffer { pass: String, resource: String },
    #[error(
        "render graph pass `{pass}` selects mip range base {base_mip_level} count {mip_level_count:?} for texture `{resource}`, but its descriptor has {mip_levels} mip levels"
    )]
    TextureAccessMipRangeOutOfBounds {
        pass: String,
        resource: String,
        base_mip_level: u32,
        mip_level_count: Option<u32>,
        mip_levels: u32,
    },
    #[error(
        "render graph pass `{pass}` selects array-layer range base {base_array_layer} count {array_layer_count:?} for texture `{resource}`, but its descriptor has {array_layers} addressable layers"
    )]
    TextureAccessArrayLayerRangeOutOfBounds {
        pass: String,
        resource: String,
        base_array_layer: u32,
        array_layer_count: Option<u32>,
        array_layers: u32,
    },
    #[error(
        "render graph pass `{pass}` selects {aspect:?} aspect for texture `{resource}`, but its descriptor format is {format:?}"
    )]
    TextureAccessAspectUnsupported {
        pass: String,
        resource: String,
        aspect: RenderGraphTextureAspect,
        format: crate::rhi::TextureFormat,
    },
    #[error("render graph pass `{pass}` selects an empty byte range for buffer `{resource}`")]
    BufferAccessRangeEmpty { pass: String, resource: String },
    #[error(
        "render graph pass `{pass}` selects byte range offset {offset} size {size:?} for buffer `{resource}`, but its descriptor has {buffer_size} bytes"
    )]
    BufferAccessRangeOutOfBounds {
        pass: String,
        resource: String,
        offset: u64,
        size: Option<u64>,
        buffer_size: u64,
    },
    #[error(
        "render graph pass `{pass}` assigns typed access intent to report-only external resource `{resource}`"
    )]
    UnresolvedExternalAccessMetadata { pass: String, resource: String },
    #[error(
        "render graph pass `{pass}` declares {declared_access:?} access for resource `{resource}`, but intent {intent:?} requires the opposite direction"
    )]
    ResourceAccessIntentKindMismatch {
        pass: String,
        resource: String,
        declared_access: RenderGraphResourceAccessKind,
        intent: RenderGraphResourceAccessIntent,
    },
    #[error(
        "render graph pass `{pass}` assigns texture intent {intent:?} to non-texture resource `{resource}`"
    )]
    ResourceAccessIntentRequiresTexture {
        pass: String,
        resource: String,
        intent: RenderGraphResourceAccessIntent,
    },
    #[error(
        "render graph pass `{pass}` assigns buffer intent {intent:?} to non-buffer resource `{resource}`"
    )]
    ResourceAccessIntentRequiresBuffer {
        pass: String,
        resource: String,
        intent: RenderGraphResourceAccessIntent,
    },
    #[error(
        "render graph pass `{pass}` assigns shader-visible intent {intent:?} to resource `{resource}` without a shader stage"
    )]
    ResourceAccessIntentShaderStagesEmpty {
        pass: String,
        resource: String,
        intent: RenderGraphResourceAccessIntent,
    },
    #[error(
        "render graph pass `{pass}` intent {intent:?} requires texture usage {required:?} for resource `{resource}`, but its descriptor declares {actual:?}"
    )]
    TextureAccessIntentUsageMissing {
        pass: String,
        resource: String,
        intent: RenderGraphResourceAccessIntent,
        required: TextureUsage,
        actual: TextureUsage,
    },
    #[error(
        "render graph pass `{pass}` intent {intent:?} requires buffer usage {required:?} for resource `{resource}`, but its descriptor declares {actual:?}"
    )]
    BufferAccessIntentUsageMissing {
        pass: String,
        resource: String,
        intent: RenderGraphResourceAccessIntent,
        required: BufferUsage,
        actual: BufferUsage,
    },
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
        "render graph compute pass `{pass}` dispatch resource `{resource}` requires a typed {required} physical descriptor"
    )]
    ComputeDispatchResourcePhysicalContractMissing {
        pass: String,
        resource: String,
        required: &'static str,
    },
    #[error(
        "render graph compute pass `{pass}` indirect dispatch resource `{resource}` requires buffer usage {required:?}, but its compiled descriptor declares {actual:?}"
    )]
    ComputeIndirectDispatchUsageMissing {
        pass: String,
        resource: String,
        required: BufferUsage,
        actual: BufferUsage,
    },
    #[error(
        "render graph compute pass `{pass}` indirect dispatch resource `{resource}` offset {offset} overflows the 12-byte command window"
    )]
    ComputeIndirectDispatchRangeOverflow {
        pass: String,
        resource: String,
        offset: u64,
    },
    #[error(
        "render graph compute pass `{pass}` indirect dispatch resource `{resource}` command at offset {offset} must declare exactly its 12-byte access range, not [{range_start}..{range_end})"
    )]
    ComputeIndirectDispatchRangeNotExact {
        pass: String,
        resource: String,
        offset: u64,
        range_start: u64,
        range_end: u64,
    },
    #[error(
        "render graph compute pass `{pass}` per-pixel dispatch target `{resource}` selected physical mip {selected_base_mip_level}, outside logical target view [{target_base_mip_level}..{target_mip_end})"
    )]
    ComputePerPixelDispatchAccessScopeOutsideTarget {
        pass: String,
        resource: String,
        selected_base_mip_level: u32,
        target_base_mip_level: u32,
        target_mip_end: u32,
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
        "render graph compute pass `{pass}` binding `{binding}` selects buffer range offset {offset} size {size:?} on a non-buffer binding"
    )]
    ComputeBufferRangeBindingNotBuffer {
        pass: String,
        binding: u32,
        offset: u64,
        size: Option<u64>,
    },
    #[error(
        "render graph compute pass `{pass}` binding `{binding}` resource `{resource}` selects an empty buffer range"
    )]
    ComputeBufferBindingRangeEmpty {
        pass: String,
        binding: u32,
        resource: String,
    },
    #[error(
        "render graph compute pass `{pass}` binding `{binding}` resource `{resource}` selects range offset {offset} size {size:?} outside its {buffer_size} byte compiled buffer descriptor"
    )]
    ComputeBufferBindingRangeOutOfBounds {
        pass: String,
        binding: u32,
        resource: String,
        offset: u64,
        size: Option<u64>,
        buffer_size: u64,
    },
    #[error(
        "render graph compute pass `{pass}` binding `{binding}` resource `{resource}` requires buffer usage {required:?}, but its compiled descriptor declares {actual:?}"
    )]
    ComputeBufferBindingUsageMissing {
        pass: String,
        binding: u32,
        resource: String,
        required: BufferUsage,
        actual: BufferUsage,
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
    #[error(
        "render graph `{graph_name}` has no present, readback, persistent, or side-effect cull root"
    )]
    MissingCullRoot { graph_name: String },
}
