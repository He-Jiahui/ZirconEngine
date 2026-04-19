use crate::render_graph::QueueLane;

use crate::graphics::pipeline::RenderPassStage;

use super::render_feature_pass_descriptor::RenderFeaturePassDescriptor;

impl RenderFeaturePassDescriptor {
    pub fn new(stage: RenderPassStage, pass_name: impl Into<String>, queue: QueueLane) -> Self {
        Self {
            stage,
            pass_name: pass_name.into(),
            queue,
        }
    }
}
