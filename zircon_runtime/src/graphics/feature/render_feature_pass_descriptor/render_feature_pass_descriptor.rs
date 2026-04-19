use crate::render_graph::QueueLane;

use crate::graphics::pipeline::RenderPassStage;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderFeaturePassDescriptor {
    pub stage: RenderPassStage,
    pub pass_name: String,
    pub queue: QueueLane,
}
