use crate::graphics::scene::RenderPassExecutorId;
use crate::render_graph::{
    PassFlags, QueueLane, RenderGraphAttachmentOps, RenderGraphComputeWorkload,
    RenderGraphExternalResourceBinding, RenderGraphResourceAccessMetadata,
    RenderGraphResourceUsageFlags, RenderGraphTextureSubresourceRange,
};

use crate::graphics::pipeline::RenderPassStage;

use super::super::compute_pass_descriptor::ComputePassDescriptor;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderFeatureResourceAccess {
    Read,
    Write,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RenderFeatureResourceKind {
    Texture,
    Buffer,
    External,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderFeatureResourceWriteMode {
    /// The write is a render-pass attachment write and may carry load/store ops.
    Attachment,
    /// The write is a storage binding write from compute or another non-attachment path.
    Storage,
}

/// A producer-owned resource value consumed by a feature pass.
///
/// Resource names describe storage identity. A version additionally names the
/// pass that produced the value, allowing reads and attachment loads to form
/// an explicit data edge without relying on registration or executor ordering.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RenderFeatureResourceVersion {
    resource_name: String,
    resource_kind: RenderFeatureResourceKind,
    producer_pass_name: String,
}

impl RenderFeatureResourceVersion {
    pub fn new(
        resource_name: impl Into<String>,
        resource_kind: RenderFeatureResourceKind,
        producer_pass_name: impl Into<String>,
    ) -> Self {
        Self {
            resource_name: resource_name.into(),
            resource_kind,
            producer_pass_name: producer_pass_name.into(),
        }
    }

    pub fn resource_name(&self) -> &str {
        &self.resource_name
    }

    pub const fn resource_kind(&self) -> RenderFeatureResourceKind {
        self.resource_kind
    }

    pub fn producer_pass_name(&self) -> &str {
        &self.producer_pass_name
    }
}

/// Product-level declaration of a logical texture resource backed by a view
/// into another graph-owned transient texture. Names are resolved once during
/// graph authoring and never used by the materializer to infer view topology.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderFeatureTextureViewAlias {
    pub parent_resource: String,
    pub range: RenderGraphTextureSubresourceRange,
}

impl RenderFeatureTextureViewAlias {
    pub fn new(
        parent_resource: impl Into<String>,
        range: RenderGraphTextureSubresourceRange,
    ) -> Self {
        Self {
            parent_resource: parent_resource.into(),
            range,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderFeatureResourceDescriptor {
    pub name: String,
    pub kind: RenderFeatureResourceKind,
    pub access: RenderFeatureResourceAccess,
    /// The precise produced value read or loaded by this access, when its
    /// producer is declared independently of feature registration order.
    pub input_version: Option<RenderFeatureResourceVersion>,
    /// Minimum allocation required by a transient buffer protocol, independent of viewport size.
    pub minimum_size_bytes: Option<u64>,
    pub attachment_ops: Option<RenderGraphAttachmentOps>,
    pub write_mode: RenderFeatureResourceWriteMode,
    /// Optional canonical scope and intent supplied by a product authoring path.
    /// Legacy feature declarations retain `None` until they opt into exact graph access APIs.
    pub access_metadata: Option<RenderGraphResourceAccessMetadata>,
    pub external_binding: RenderGraphExternalResourceBinding,
    /// Declares this resource as a graph-owned view of a parent texture.
    /// Only the definition access needs this metadata; later accesses retain
    /// the ordinary logical resource name.
    pub texture_view_alias: Option<RenderFeatureTextureViewAlias>,
    /// Typed allocation contract for textures whose physical shape cannot be
    /// supplied by a built-in product resource catalog.
    pub schema: Option<super::RenderResourceSchema>,
    /// Explicit graph lifetime role. Roles are authored by the feature that
    /// owns the terminal output or cross-frame resource, never inferred from
    /// a resource name during graph compilation.
    pub usage: RenderGraphResourceUsageFlags,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderFeaturePassDescriptor {
    pub stage: RenderPassStage,
    pub pass_name: String,
    pub queue: QueueLane,
    pub flags: PassFlags,
    pub executor_id: RenderPassExecutorId,
    pub compute_workload: Option<RenderGraphComputeWorkload>,
    pub compute_pass: Option<ComputePassDescriptor>,
    pub resources: Vec<RenderFeatureResourceDescriptor>,
}
