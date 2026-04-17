use zircon_render_graph::QueueLane;

use crate::pipeline::RenderPassStage;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderFeaturePassDescriptor {
    pub stage: RenderPassStage,
    pub pass_name: String,
    pub queue: QueueLane,
}
