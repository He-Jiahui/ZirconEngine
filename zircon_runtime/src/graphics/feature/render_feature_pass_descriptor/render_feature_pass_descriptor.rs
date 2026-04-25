use crate::graphics::scene::RenderPassExecutorId;
use crate::render_graph::{PassFlags, QueueLane};

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderFeatureResourceDescriptor {
    pub name: String,
    pub kind: RenderFeatureResourceKind,
    pub access: RenderFeatureResourceAccess,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderFeaturePassDescriptor {
    pub stage: RenderPassStage,
    pub pass_name: String,
    pub queue: QueueLane,
    pub flags: PassFlags,
    pub executor_id: RenderPassExecutorId,
    pub resources: Vec<RenderFeatureResourceDescriptor>,
}
