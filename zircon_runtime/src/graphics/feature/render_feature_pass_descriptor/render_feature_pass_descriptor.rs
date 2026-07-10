use crate::graphics::scene::RenderPassExecutorId;
use crate::render_graph::{
    PassFlags, QueueLane, RenderGraphAttachmentOps, RenderGraphComputeWorkload,
    RenderGraphExternalResourceBinding,
};

use crate::graphics::pipeline::RenderPassStage;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderFeatureResourceAccess {
    Read,
    Write,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderFeatureResourceDescriptor {
    pub name: String,
    pub kind: RenderFeatureResourceKind,
    pub access: RenderFeatureResourceAccess,
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
    pub resources: Vec<RenderFeatureResourceDescriptor>,
}
