use crate::graphics::scene::RenderPassExecutorId;
use crate::render_graph::{
    PassFlags, QueueLane, RenderGraphAttachmentOps, RenderGraphComputeWorkload,
    RenderGraphExternalResourceBinding,
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
/// pass that produced the value, allowing feature descriptors to form an
/// explicit data edge without relying on registration or executor ordering.
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderFeatureResourceDescriptor {
    pub name: String,
    pub kind: RenderFeatureResourceKind,
    pub access: RenderFeatureResourceAccess,
    /// The precise produced value read by this access, when its producer is
    /// declared independently of feature registration order.
    pub input_version: Option<RenderFeatureResourceVersion>,
    /// Minimum allocation required by a transient buffer protocol, independent of viewport size.
    pub minimum_size_bytes: Option<u64>,
    pub attachment_ops: Option<RenderGraphAttachmentOps>,
    pub write_mode: RenderFeatureResourceWriteMode,
    pub external_binding: RenderGraphExternalResourceBinding,
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
