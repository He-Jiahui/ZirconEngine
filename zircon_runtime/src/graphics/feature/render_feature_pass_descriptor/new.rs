use crate::graphics::scene::RenderPassExecutorId;
use crate::render_graph::QueueLane;

use crate::graphics::pipeline::RenderPassStage;

use super::render_feature_pass_descriptor::{
    RenderFeaturePassDescriptor, RenderFeatureResourceAccess, RenderFeatureResourceDescriptor,
    RenderFeatureResourceKind,
};

impl RenderFeaturePassDescriptor {
    pub fn new(stage: RenderPassStage, pass_name: impl Into<String>, queue: QueueLane) -> Self {
        let pass_name = pass_name.into();
        Self {
            stage,
            executor_id: RenderPassExecutorId::new(pass_name.clone()),
            pass_name,
            queue,
            flags: Default::default(),
            resources: Vec::new(),
        }
    }

    pub fn with_executor_id(mut self, executor_id: impl Into<RenderPassExecutorId>) -> Self {
        self.executor_id = executor_id.into();
        self
    }

    pub fn with_side_effects(mut self) -> Self {
        self.flags.has_side_effects = true;
        self
    }

    pub fn read_texture(self, name: impl Into<String>) -> Self {
        self.with_resource(
            name,
            RenderFeatureResourceKind::Texture,
            RenderFeatureResourceAccess::Read,
        )
    }

    pub fn write_texture(self, name: impl Into<String>) -> Self {
        self.with_resource(
            name,
            RenderFeatureResourceKind::Texture,
            RenderFeatureResourceAccess::Write,
        )
    }

    pub fn read_buffer(self, name: impl Into<String>) -> Self {
        self.with_resource(
            name,
            RenderFeatureResourceKind::Buffer,
            RenderFeatureResourceAccess::Read,
        )
    }

    pub fn write_buffer(self, name: impl Into<String>) -> Self {
        self.with_resource(
            name,
            RenderFeatureResourceKind::Buffer,
            RenderFeatureResourceAccess::Write,
        )
    }

    pub fn read_external(self, name: impl Into<String>) -> Self {
        self.with_resource(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Read,
        )
    }

    pub fn write_external(self, name: impl Into<String>) -> Self {
        self.with_resource(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Write,
        )
    }

    fn with_resource(
        mut self,
        name: impl Into<String>,
        kind: RenderFeatureResourceKind,
        access: RenderFeatureResourceAccess,
    ) -> Self {
        self.resources.push(RenderFeatureResourceDescriptor {
            name: name.into(),
            kind,
            access,
        });
        self
    }
}
