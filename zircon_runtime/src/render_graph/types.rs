use crate::rhi::{BufferDesc, TextureDesc};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RenderPassId(pub(crate) usize);

/// Stable logical texture handle allocated by `RenderGraphBuilder`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RgTextureHandle(pub(crate) usize);

impl RgTextureHandle {
    pub(crate) const fn from_index(index: usize) -> Self {
        Self(index)
    }

    pub const fn index(self) -> usize {
        self.0
    }
}

/// Stable logical buffer handle allocated by `RenderGraphBuilder`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RgBufferHandle(pub(crate) usize);

impl RgBufferHandle {
    pub(crate) const fn from_index(index: usize) -> Self {
        Self(index)
    }

    pub const fn index(self) -> usize {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ExternalResource(pub(crate) usize);

impl ExternalResource {
    pub(crate) const fn from_index(index: usize) -> Self {
        Self(index)
    }

    pub const fn index(self) -> usize {
        self.0
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RenderGraphResourceKind {
    TransientTexture,
    TransientBuffer,
    External,
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
    pub first_pass: usize,
    pub last_pass: usize,
    pub imported: bool,
}

impl RenderGraphResourceLifetime {
    pub fn is_sparse_reserved_texture(&self) -> bool {
        matches!(
            &self.desc,
            RenderGraphResourceDesc::Texture(desc) if desc.is_sparse_reserved()
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderGraphResourceAccessKind {
    Read,
    Write,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderGraphAttachmentLoadOp {
    Load,
    Clear,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    pub imported: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderGraphPassResourceAccess {
    pub name: String,
    pub kind: RenderGraphResourceKind,
    pub access: RenderGraphResourceAccessKind,
    pub attachment_ops: Option<RenderGraphAttachmentOps>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RenderGraphComputeDispatchExtent {
    Viewport,
    ClusterGrid,
    HzbFurthest,
    Fixed([u32; 3]),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderGraphComputeWorkload {
    /// Neutral planned workload metadata. Concrete WGPU pipelines, bind groups, and
    /// dispatch recording stay owned by renderer executors.
    pub pipeline_label: String,
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
            workgroup_size,
            dispatch_extent,
        }
    }

    pub fn viewport(pipeline_label: impl Into<String>, workgroup_size: [u32; 3]) -> Self {
        Self::new(
            pipeline_label,
            workgroup_size,
            RenderGraphComputeDispatchExtent::Viewport,
        )
    }

    pub fn cluster_grid(pipeline_label: impl Into<String>, workgroup_size: [u32; 3]) -> Self {
        Self::new(
            pipeline_label,
            workgroup_size,
            RenderGraphComputeDispatchExtent::ClusterGrid,
        )
    }

    pub fn hzb_furthest(pipeline_label: impl Into<String>, workgroup_size: [u32; 3]) -> Self {
        Self::new(
            pipeline_label,
            workgroup_size,
            RenderGraphComputeDispatchExtent::HzbFurthest,
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
