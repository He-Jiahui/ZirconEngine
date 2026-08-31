use crate::core::framework::render::{ComputeDispatchPlan, ShaderDispatchExtent};
use crate::rhi::{BufferDesc, TextureDesc};
use zircon_runtime_interface::resource::AssetReference;

use super::access::{
    RenderGraphBufferRange, RenderGraphResourceAccessIntent, RenderGraphResourceAccessMetadata,
    RenderGraphResourceAccessRange, RenderGraphShaderStages, RenderGraphTextureSubresourceRange,
};

mod compute_pipeline;

pub use compute_pipeline::{
    RenderGraphComputePipelineFallbackPolicy, RenderGraphComputePipelineFamily,
    RenderGraphComputePipelineResolution, RenderGraphComputePipelineResolutionStatus,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RenderPassId(pub(crate) usize, pub(crate) u64);

impl RenderPassId {
    pub(crate) const fn from_index(index: usize, generation: u64) -> Self {
        Self(index, generation)
    }

    pub const fn index(self) -> usize {
        self.0
    }

    pub(crate) const fn generation(self) -> u64 {
        self.1
    }
}

/// Builder-scoped logical texture handle allocated by `RenderGraphBuilder`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RgTextureHandle(pub(crate) usize, pub(crate) u64);

impl RgTextureHandle {
    pub(crate) const fn from_index(index: usize, generation: u64) -> Self {
        Self(index, generation)
    }

    pub const fn index(self) -> usize {
        self.0
    }

    pub(crate) const fn generation(self) -> u64 {
        self.1
    }
}

/// A logical texture resource backed by an exact subresource view of another
/// graph-owned transient texture. The alias retains a distinct handle so
/// accesses, versions, and culling keep their logical identity while physical
/// allocation remains owned by `parent`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RenderGraphTextureViewAlias {
    pub parent: RgTextureHandle,
    pub range: RenderGraphTextureSubresourceRange,
}

impl RenderGraphTextureViewAlias {
    pub const fn new(parent: RgTextureHandle, range: RenderGraphTextureSubresourceRange) -> Self {
        Self { parent, range }
    }
}

/// Builder-scoped logical buffer handle allocated by `RenderGraphBuilder`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RgBufferHandle(pub(crate) usize, pub(crate) u64);

impl RgBufferHandle {
    pub(crate) const fn from_index(index: usize, generation: u64) -> Self {
        Self(index, generation)
    }

    pub const fn index(self) -> usize {
        self.0
    }

    pub(crate) const fn generation(self) -> u64 {
        self.1
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ExternalResource(pub(crate) usize, pub(crate) u64);

impl ExternalResource {
    pub(crate) const fn from_index(index: usize, generation: u64) -> Self {
        Self(index, generation)
    }

    pub const fn index(self) -> usize {
        self.0
    }

    pub(crate) const fn generation(self) -> u64 {
        self.1
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RenderGraphResource {
    TransientTexture(RgTextureHandle),
    TransientBuffer(RgBufferHandle),
    External(ExternalResource),
}

impl RenderGraphResource {
    pub const fn kind(self) -> RenderGraphResourceKind {
        match self {
            Self::TransientTexture(_) => RenderGraphResourceKind::TransientTexture,
            Self::TransientBuffer(_) => RenderGraphResourceKind::TransientBuffer,
            Self::External(_) => RenderGraphResourceKind::External,
        }
    }
}

/// Immutable logical value identity produced by a render graph resource write.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RenderGraphResourceVersion {
    resource: RenderGraphResource,
    ordinal: u64,
}

impl RenderGraphResourceVersion {
    pub(crate) const fn new(resource: RenderGraphResource, ordinal: u64) -> Self {
        Self { resource, ordinal }
    }

    pub const fn resource(self) -> RenderGraphResource {
        self.resource
    }

    pub const fn ordinal(self) -> u64 {
        self.ordinal
    }
}

/// Builder-scoped reference to the exact write access that produced a logical value.
///
/// This is an authoring token rather than a compiled version ordinal. The compiler
/// resolves it after scheduling and rejects a consumer if another write superseded
/// the requested value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RenderGraphResourceVersionToken {
    resource: RenderGraphResource,
    producer_pass: RenderPassId,
    producer_access_index: usize,
    builder_generation: u64,
}

impl RenderGraphResourceVersionToken {
    pub(crate) const fn new(
        resource: RenderGraphResource,
        producer_pass: RenderPassId,
        producer_access_index: usize,
        builder_generation: u64,
    ) -> Self {
        Self {
            resource,
            producer_pass,
            producer_access_index,
            builder_generation,
        }
    }

    pub(crate) const fn resource(self) -> RenderGraphResource {
        self.resource
    }

    pub(crate) const fn producer_pass(self) -> RenderPassId {
        self.producer_pass
    }

    pub(crate) const fn producer_access_index(self) -> usize {
        self.producer_access_index
    }

    pub(crate) const fn builder_generation(self) -> u64 {
        self.builder_generation
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RenderGraphResourceKind {
    TransientTexture,
    TransientBuffer,
    External,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum RenderGraphExternalResourceType {
    #[default]
    Unknown,
    Texture,
    Buffer,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum RenderGraphExternalResourceRequirement {
    #[default]
    ReportOnly,
    Required,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct RenderGraphExternalResourceBinding {
    pub resource_type: RenderGraphExternalResourceType,
    pub requirement: RenderGraphExternalResourceRequirement,
}

impl RenderGraphExternalResourceBinding {
    pub const fn report_only() -> Self {
        Self {
            resource_type: RenderGraphExternalResourceType::Unknown,
            requirement: RenderGraphExternalResourceRequirement::ReportOnly,
        }
    }

    pub const fn report_only_texture() -> Self {
        Self {
            resource_type: RenderGraphExternalResourceType::Texture,
            requirement: RenderGraphExternalResourceRequirement::ReportOnly,
        }
    }

    pub const fn report_only_buffer() -> Self {
        Self {
            resource_type: RenderGraphExternalResourceType::Buffer,
            requirement: RenderGraphExternalResourceRequirement::ReportOnly,
        }
    }

    pub const fn required_buffer() -> Self {
        Self {
            resource_type: RenderGraphExternalResourceType::Buffer,
            requirement: RenderGraphExternalResourceRequirement::Required,
        }
    }

    pub const fn required_texture() -> Self {
        Self {
            resource_type: RenderGraphExternalResourceType::Texture,
            requirement: RenderGraphExternalResourceRequirement::Required,
        }
    }

    pub const fn is_required(self) -> bool {
        matches!(
            self.requirement,
            RenderGraphExternalResourceRequirement::Required
        )
    }

    pub const fn label(self) -> &'static str {
        match self.resource_type {
            RenderGraphExternalResourceType::Unknown => "external",
            RenderGraphExternalResourceType::Texture => "external texture",
            RenderGraphExternalResourceType::Buffer => "external buffer",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct RenderGraphResourceUsageFlags {
    pub present: bool,
    pub readback: bool,
    pub persistent: bool,
}

impl RenderGraphResourceUsageFlags {
    pub const fn present() -> Self {
        Self {
            present: true,
            readback: false,
            persistent: false,
        }
    }

    pub const fn readback() -> Self {
        Self {
            present: false,
            readback: true,
            persistent: false,
        }
    }

    pub const fn persistent() -> Self {
        Self {
            present: false,
            readback: false,
            persistent: true,
        }
    }

    pub const fn is_cull_root(self) -> bool {
        self.present || self.readback || self.persistent
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RenderGraphResourceDesc {
    Texture(TextureDesc),
    Buffer(BufferDesc),
    External,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderGraphResourceLifetime {
    pub resource: RenderGraphResource,
    pub name: String,
    pub kind: RenderGraphResourceKind,
    pub desc: RenderGraphResourceDesc,
    pub external_binding: RenderGraphExternalResourceBinding,
    /// Resolved RHI contract for a schema-backed imported texture.
    ///
    /// Generic report-only imports intentionally remain descriptor-less, but a
    /// feature that declares a texture schema must carry its resolved shape to
    /// execution so the bound physical lease can be verified before encoding.
    pub external_texture_desc: Option<TextureDesc>,
    /// Resolved RHI contract for a schema-backed imported buffer.
    pub external_buffer_desc: Option<BufferDesc>,
    /// The declared parent texture view for this logical resource, when it
    /// owns no physical allocation of its own.
    pub texture_view_alias: Option<RenderGraphTextureViewAlias>,
    pub first_pass: usize,
    pub last_pass: usize,
    pub imported: bool,
    pub usage: RenderGraphResourceUsageFlags,
}

impl RenderGraphResourceLifetime {
    pub fn is_sparse_reserved_texture(&self) -> bool {
        matches!(
            &self.desc,
            RenderGraphResourceDesc::Texture(desc) if desc.is_sparse_reserved()
        )
    }

    pub const fn is_texture_view_alias(&self) -> bool {
        self.texture_view_alias.is_some()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RenderGraphResourceAccessKind {
    Read,
    Write,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderGraphAttachmentLoadOp {
    Load,
    Clear,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RenderGraphAttachmentStoreOp {
    Store,
    Discard,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderGraphAttachmentOps {
    pub load: RenderGraphAttachmentLoadOp,
    pub store: RenderGraphAttachmentStoreOp,
}

impl RenderGraphAttachmentOps {
    pub const fn clear_store() -> Self {
        Self {
            load: RenderGraphAttachmentLoadOp::Clear,
            store: RenderGraphAttachmentStoreOp::Store,
        }
    }

    pub const fn load_store() -> Self {
        Self {
            load: RenderGraphAttachmentLoadOp::Load,
            store: RenderGraphAttachmentStoreOp::Store,
        }
    }

    pub const fn clear_discard() -> Self {
        Self {
            load: RenderGraphAttachmentLoadOp::Clear,
            store: RenderGraphAttachmentStoreOp::Discard,
        }
    }
}

/// Immutable declaration row for a graph resource, kept even if all uses are culled.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderGraphResourceDeclaration {
    pub resource: RenderGraphResource,
    pub name: String,
    pub kind: RenderGraphResourceKind,
    pub desc: RenderGraphResourceDesc,
    pub external_binding: RenderGraphExternalResourceBinding,
    /// Resolved RHI contract for a schema-backed imported texture.
    pub external_texture_desc: Option<TextureDesc>,
    /// Resolved RHI contract for a schema-backed imported buffer.
    pub external_buffer_desc: Option<BufferDesc>,
    /// Optional graph-owned subresource view backing this logical texture.
    pub texture_view_alias: Option<RenderGraphTextureViewAlias>,
    pub imported: bool,
    pub usage: RenderGraphResourceUsageFlags,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderGraphPassResourceAccess {
    pub name: String,
    pub kind: RenderGraphResourceKind,
    pub access: RenderGraphResourceAccessKind,
    pub attachment_ops: Option<RenderGraphAttachmentOps>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComputeBindingKind {
    UniformBuffer,
    StorageBufferRead,
    StorageBufferReadWrite,
    SampledTexture,
    StorageTextureWrite,
}

/// A static byte window for a graph buffer binding.
///
/// Dynamic offsets are command-recording state and deliberately do not belong
/// to immutable render-graph metadata.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RenderGraphBufferBindingRange {
    pub offset: u64,
    pub size: Option<u64>,
}

impl RenderGraphBufferBindingRange {
    pub const fn new(offset: u64, size: Option<u64>) -> Self {
        Self { offset, size }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BindingSchemaEntry {
    pub binding: u32,
    pub resource: String,
    pub kind: ComputeBindingKind,
    /// `None` binds the resource's default view; `Some` binds exactly one owned texture mip.
    pub texture_mip_level: Option<u32>,
    /// Requests an owned texture view spanning all mips when available; imported textures retain
    /// their default view because their backing mip topology is not owned by the render graph.
    pub texture_full_mip_chain: bool,
    /// `None` binds the whole buffer; `Some` binds the exact static byte window.
    pub buffer_range: Option<RenderGraphBufferBindingRange>,
}

impl BindingSchemaEntry {
    pub fn new(binding: u32, resource: impl Into<String>, kind: ComputeBindingKind) -> Self {
        Self {
            binding,
            resource: resource.into(),
            kind,
            texture_mip_level: None,
            texture_full_mip_chain: false,
            buffer_range: None,
        }
    }

    pub fn with_texture_mip_level(mut self, mip_level: u32) -> Self {
        self.texture_mip_level = Some(mip_level);
        self.texture_full_mip_chain = false;
        self
    }

    pub fn with_texture_full_mip_chain(mut self) -> Self {
        self.texture_mip_level = None;
        self.texture_full_mip_chain = true;
        self
    }

    pub fn with_buffer_range(mut self, offset: u64, size: Option<u64>) -> Self {
        self.buffer_range = Some(RenderGraphBufferBindingRange::new(offset, size));
        self
    }

    /// Returns the graph scope and intent required for one logical access of this binding.
    ///
    /// `StorageBufferReadWrite` intentionally yields distinct read and write metadata so the
    /// compiler can retain both provenance rows rather than assigning one mutable binding a
    /// fictitious single access identity.
    pub(crate) fn compute_access_metadata(
        &self,
        access: RenderGraphResourceAccessKind,
    ) -> Option<RenderGraphResourceAccessMetadata> {
        let stages = RenderGraphShaderStages::COMPUTE;
        let buffer_range = self
            .buffer_range
            .map(|range| RenderGraphBufferRange::new(range.offset, range.size))
            .unwrap_or_else(RenderGraphBufferRange::full);
        let texture_range = self
            .texture_mip_level
            .map(RenderGraphTextureSubresourceRange::single_mip)
            .unwrap_or_else(RenderGraphTextureSubresourceRange::full);
        let intent = match (self.kind, access) {
            (ComputeBindingKind::UniformBuffer, RenderGraphResourceAccessKind::Read) => {
                RenderGraphResourceAccessIntent::UniformBuffer { stages }
            }
            (ComputeBindingKind::StorageBufferRead, RenderGraphResourceAccessKind::Read)
            | (ComputeBindingKind::StorageBufferReadWrite, RenderGraphResourceAccessKind::Read) => {
                RenderGraphResourceAccessIntent::storage_buffer_read(stages)
            }
            (ComputeBindingKind::StorageBufferReadWrite, RenderGraphResourceAccessKind::Write) => {
                RenderGraphResourceAccessIntent::storage_buffer_read_write(stages)
            }
            (ComputeBindingKind::SampledTexture, RenderGraphResourceAccessKind::Read) => {
                RenderGraphResourceAccessIntent::sampled_texture(stages)
            }
            (ComputeBindingKind::StorageTextureWrite, RenderGraphResourceAccessKind::Write) => {
                RenderGraphResourceAccessIntent::storage_texture_write(stages)
            }
            _ => return None,
        };
        let range = match self.kind {
            ComputeBindingKind::UniformBuffer
            | ComputeBindingKind::StorageBufferRead
            | ComputeBindingKind::StorageBufferReadWrite => {
                RenderGraphResourceAccessRange::Buffer(buffer_range)
            }
            ComputeBindingKind::SampledTexture | ComputeBindingKind::StorageTextureWrite => {
                RenderGraphResourceAccessRange::Texture(texture_range)
            }
        };
        Some(RenderGraphResourceAccessMetadata::new(range, intent))
    }
}

/// Graph-owned execution payload lowered from a graphics compute descriptor.
/// It deliberately contains no WGPU objects so compiled graphs stay reusable.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RenderGraphComputeShaderSource {
    Wgsl { label: String, source: String },
    Asset { asset: AssetReference },
}

impl RenderGraphComputeShaderSource {
    pub fn wgsl(label: impl Into<String>, source: impl Into<String>) -> Self {
        Self::Wgsl {
            label: label.into(),
            source: source.into(),
        }
    }

    pub fn asset(asset: AssetReference) -> Self {
        Self::Asset { asset }
    }
}

/// The concrete compute payload paired with a pass workload in a compiled graph.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderGraphComputePassMetadata {
    pub shader: RenderGraphComputeShaderSource,
    pub entry_point: String,
    pub bindings: Vec<BindingSchemaEntry>,
}

impl RenderGraphComputePassMetadata {
    pub fn new(
        shader: RenderGraphComputeShaderSource,
        entry_point: impl Into<String>,
        bindings: Vec<BindingSchemaEntry>,
    ) -> Self {
        Self {
            shader,
            entry_point: entry_point.into(),
            bindings,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RenderGraphComputeDispatchExtent {
    ClusterGrid,
    FroxelGrid,
    FroxelGridXy,
    HzbFurthest,
    IndirectArgs,
    Fixed([u32; 3]),
    FromBuffer {
        buffer: String,
        offset: u64,
    },
    PerPixel {
        target: String,
        local_size: [u32; 2],
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderGraphComputeWorkload {
    /// Neutral planned workload metadata. Concrete WGPU pipelines, bind groups, and
    /// dispatch recording stay owned by renderer executors.
    pub pipeline_label: String,
    pub pipeline_fallback_policy: RenderGraphComputePipelineFallbackPolicy,
    pub workgroup_size: [u32; 3],
    pub dispatch_extent: RenderGraphComputeDispatchExtent,
}

impl RenderGraphComputeWorkload {
    pub fn new(
        pipeline_label: impl Into<String>,
        workgroup_size: [u32; 3],
        dispatch_extent: RenderGraphComputeDispatchExtent,
    ) -> Self {
        Self {
            pipeline_label: pipeline_label.into(),
            pipeline_fallback_policy: RenderGraphComputePipelineFallbackPolicy::Reject,
            workgroup_size,
            dispatch_extent,
        }
    }

    pub fn with_pipeline_fallback_policy(
        mut self,
        policy: RenderGraphComputePipelineFallbackPolicy,
    ) -> Self {
        self.pipeline_fallback_policy = policy;
        self
    }

    pub fn cluster_grid(pipeline_label: impl Into<String>, workgroup_size: [u32; 3]) -> Self {
        Self::new(
            pipeline_label,
            workgroup_size,
            RenderGraphComputeDispatchExtent::ClusterGrid,
        )
    }

    pub fn froxel_grid(pipeline_label: impl Into<String>, workgroup_size: [u32; 3]) -> Self {
        Self::new(
            pipeline_label,
            workgroup_size,
            RenderGraphComputeDispatchExtent::FroxelGrid,
        )
    }

    pub fn froxel_grid_xy(pipeline_label: impl Into<String>, workgroup_size: [u32; 3]) -> Self {
        Self::new(
            pipeline_label,
            workgroup_size,
            RenderGraphComputeDispatchExtent::FroxelGridXy,
        )
    }

    pub fn hzb_furthest(pipeline_label: impl Into<String>, workgroup_size: [u32; 3]) -> Self {
        Self::new(
            pipeline_label,
            workgroup_size,
            RenderGraphComputeDispatchExtent::HzbFurthest,
        )
    }

    pub fn indirect_args(pipeline_label: impl Into<String>, workgroup_size: [u32; 3]) -> Self {
        Self::new(
            pipeline_label,
            workgroup_size,
            RenderGraphComputeDispatchExtent::IndirectArgs,
        )
    }

    pub fn fixed(
        pipeline_label: impl Into<String>,
        workgroup_size: [u32; 3],
        dispatch_groups: [u32; 3],
    ) -> Self {
        Self::new(
            pipeline_label,
            workgroup_size,
            RenderGraphComputeDispatchExtent::Fixed(dispatch_groups),
        )
    }

    pub fn from_buffer(
        pipeline_label: impl Into<String>,
        workgroup_size: [u32; 3],
        buffer: impl Into<String>,
        offset: u64,
    ) -> Self {
        Self::new(
            pipeline_label,
            workgroup_size,
            RenderGraphComputeDispatchExtent::FromBuffer {
                buffer: buffer.into(),
                offset,
            },
        )
    }

    pub fn per_pixel(
        pipeline_label: impl Into<String>,
        workgroup_size: [u32; 3],
        target: impl Into<String>,
        local_size: [u32; 2],
    ) -> Self {
        Self::new(
            pipeline_label,
            workgroup_size,
            RenderGraphComputeDispatchExtent::PerPixel {
                target: target.into(),
                local_size,
            },
        )
    }

    pub fn from_shader_dispatch(dispatch: &ComputeDispatchPlan) -> Self {
        Self::new(
            dispatch.pipeline_label.clone(),
            dispatch.workgroup_size,
            render_graph_dispatch_extent(dispatch.dispatch_extent),
        )
    }
}

fn render_graph_dispatch_extent(extent: ShaderDispatchExtent) -> RenderGraphComputeDispatchExtent {
    match extent {
        ShaderDispatchExtent::ClusterGrid => RenderGraphComputeDispatchExtent::ClusterGrid,
        ShaderDispatchExtent::HzbFurthest => RenderGraphComputeDispatchExtent::HzbFurthest,
        ShaderDispatchExtent::IndirectArgs => RenderGraphComputeDispatchExtent::IndirectArgs,
        ShaderDispatchExtent::Fixed(groups) => RenderGraphComputeDispatchExtent::Fixed(groups),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueueLane {
    Graphics,
    AsyncCompute,
    AsyncCopy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PassFlags {
    pub allow_culling: bool,
    pub has_side_effects: bool,
}

impl Default for PassFlags {
    fn default() -> Self {
        Self {
            allow_culling: true,
            has_side_effects: false,
        }
    }
}
