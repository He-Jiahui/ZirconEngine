use crate::render_graph::{
    PassFlags, QueueLane, RenderGraphAttachmentOps, RenderGraphPassResourceAccess,
    RenderGraphResourceAccessKind, RenderGraphResourceKind, RenderPassId,
};

use super::RenderPassExecutorId;

mod gpu;

pub use gpu::RenderPassGpuExecutionContext;
pub(in crate::graphics::scene::scene_renderer) use gpu::{
    RenderPassMeshDrawLists, RenderPassPostProcessStackContext,
};

pub struct RenderPassExecutionContext<'a> {
    pub pass_name: String,
    pub executor_id: RenderPassExecutorId,
    pub declared_queue: QueueLane,
    pub queue: QueueLane,
    pub flags: PassFlags,
    pub dependencies: Vec<RenderPassId>,
    pub resources: Vec<RenderGraphPassResourceAccess>,
    gpu: Option<RenderPassGpuExecutionContext<'a>>,
}

impl std::fmt::Debug for RenderPassExecutionContext<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RenderPassExecutionContext")
            .field("pass_name", &self.pass_name)
            .field("executor_id", &self.executor_id)
            .field("declared_queue", &self.declared_queue)
            .field("queue", &self.queue)
            .field("flags", &self.flags)
            .field("dependencies", &self.dependencies)
            .field("resources", &self.resources)
            .field("has_gpu", &self.gpu.is_some())
            .finish()
    }
}

impl<'a> RenderPassExecutionContext<'a> {
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn new(pass_name: impl Into<String>, executor_id: RenderPassExecutorId) -> Self {
        Self::with_graph_metadata(
            pass_name,
            executor_id,
            QueueLane::Graphics,
            PassFlags::default(),
        )
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn with_graph_metadata(
        pass_name: impl Into<String>,
        executor_id: RenderPassExecutorId,
        queue: QueueLane,
        flags: PassFlags,
    ) -> Self {
        Self::with_graph_metadata_and_resources(pass_name, executor_id, queue, flags, Vec::new())
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn with_declared_graph_metadata(
        pass_name: impl Into<String>,
        executor_id: RenderPassExecutorId,
        queue: QueueLane,
        declared_queue: QueueLane,
        flags: PassFlags,
    ) -> Self {
        Self::with_declared_graph_metadata_and_resources(
            pass_name,
            executor_id,
            queue,
            declared_queue,
            flags,
            Vec::new(),
        )
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn with_graph_metadata_and_resources(
        pass_name: impl Into<String>,
        executor_id: RenderPassExecutorId,
        queue: QueueLane,
        flags: PassFlags,
        resources: Vec<RenderGraphPassResourceAccess>,
    ) -> Self {
        Self::with_declared_graph_metadata_and_resources(
            pass_name,
            executor_id,
            queue,
            queue,
            flags,
            resources,
        )
    }

    pub fn with_declared_graph_metadata_and_resources(
        pass_name: impl Into<String>,
        executor_id: RenderPassExecutorId,
        queue: QueueLane,
        declared_queue: QueueLane,
        flags: PassFlags,
        resources: Vec<RenderGraphPassResourceAccess>,
    ) -> Self {
        Self::with_declared_graph_metadata_dependencies_and_resources(
            pass_name,
            executor_id,
            queue,
            declared_queue,
            flags,
            Vec::new(),
            resources,
        )
    }

    pub fn with_declared_graph_metadata_dependencies_and_resources(
        pass_name: impl Into<String>,
        executor_id: RenderPassExecutorId,
        queue: QueueLane,
        declared_queue: QueueLane,
        flags: PassFlags,
        dependencies: Vec<RenderPassId>,
        resources: Vec<RenderGraphPassResourceAccess>,
    ) -> Self {
        Self {
            pass_name: pass_name.into(),
            executor_id,
            declared_queue,
            queue,
            flags,
            dependencies,
            resources,
            gpu: None,
        }
    }

    pub fn with_gpu(mut self, gpu: RenderPassGpuExecutionContext<'a>) -> Self {
        self.gpu = Some(gpu);
        self
    }

    pub fn gpu(&self) -> Option<&RenderPassGpuExecutionContext<'a>> {
        self.gpu.as_ref()
    }

    pub fn gpu_mut(&mut self) -> Option<&mut RenderPassGpuExecutionContext<'a>> {
        self.gpu.as_mut()
    }

    pub fn require_gpu(&mut self) -> Result<&mut RenderPassGpuExecutionContext<'a>, String> {
        self.gpu.as_mut().ok_or_else(|| {
            format!(
                "render pass executor `{}` for pass `{}` requires renderer GPU context",
                self.executor_id, self.pass_name
            )
        })
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn uses_queue_fallback(&self) -> bool {
        self.declared_queue != self.queue
    }

    pub fn attachment_ops_for_write(
        &self,
        resource_name: &str,
    ) -> Option<RenderGraphAttachmentOps> {
        self.resources
            .iter()
            .find(|resource| {
                resource.name == resource_name
                    && resource.access == RenderGraphResourceAccessKind::Write
                    && matches!(
                        resource.kind,
                        RenderGraphResourceKind::TransientTexture
                            | RenderGraphResourceKind::External
                    )
            })
            .and_then(|resource| resource.attachment_ops)
    }

    pub fn reads_texture(&self, resource_name: &str) -> bool {
        self.resources.iter().any(|resource| {
            resource.name == resource_name
                && resource.access == RenderGraphResourceAccessKind::Read
                && matches!(
                    resource.kind,
                    RenderGraphResourceKind::TransientTexture | RenderGraphResourceKind::External
                )
        })
    }

    pub fn reads_transient_texture(&self, resource_name: &str) -> bool {
        self.resources.iter().any(|resource| {
            resource.name == resource_name
                && resource.access == RenderGraphResourceAccessKind::Read
                && resource.kind == RenderGraphResourceKind::TransientTexture
        })
    }
}

#[cfg(test)]
mod tests {
    use super::RenderPassExecutionContext;
    use crate::graphics::RenderPassExecutorId;
    use crate::render_graph::{
        QueueLane, RenderGraphAttachmentOps, RenderGraphPassResourceAccess,
        RenderGraphResourceAccessKind, RenderGraphResourceKind,
    };

    #[test]
    fn metadata_context_reports_missing_gpu_payload() {
        let mut context = RenderPassExecutionContext::new(
            "particle-render",
            RenderPassExecutorId::new("particle.transparent"),
        );

        assert!(context.gpu().is_none());
        assert_eq!(
            context.require_gpu().unwrap_err(),
            "render pass executor `particle.transparent` for pass `particle-render` requires renderer GPU context"
        );
    }

    #[test]
    fn metadata_context_exposes_attachment_ops_for_written_resource() {
        let context = RenderPassExecutionContext::with_graph_metadata_and_resources(
            "transparent-mesh",
            RenderPassExecutorId::new("mesh.transparent"),
            QueueLane::Graphics,
            Default::default(),
            vec![
                RenderGraphPassResourceAccess {
                    name: "scene-color".to_string(),
                    kind: RenderGraphResourceKind::TransientTexture,
                    access: RenderGraphResourceAccessKind::Read,
                    attachment_ops: None,
                },
                RenderGraphPassResourceAccess {
                    name: "scene-color".to_string(),
                    kind: RenderGraphResourceKind::TransientTexture,
                    access: RenderGraphResourceAccessKind::Write,
                    attachment_ops: Some(RenderGraphAttachmentOps::load_store()),
                },
            ],
        );

        assert_eq!(
            context.attachment_ops_for_write("scene-color"),
            Some(RenderGraphAttachmentOps::load_store())
        );
        assert_eq!(context.attachment_ops_for_write("scene-depth"), None);
    }

    #[test]
    fn metadata_context_reports_declared_texture_reads() {
        let context = RenderPassExecutionContext::with_graph_metadata_and_resources(
            "opaque-mesh",
            RenderPassExecutorId::new("mesh.opaque"),
            QueueLane::Graphics,
            Default::default(),
            vec![
                RenderGraphPassResourceAccess {
                    name: "shadow-map".to_string(),
                    kind: RenderGraphResourceKind::TransientTexture,
                    access: RenderGraphResourceAccessKind::Read,
                    attachment_ops: None,
                },
                RenderGraphPassResourceAccess {
                    name: "scene-color".to_string(),
                    kind: RenderGraphResourceKind::TransientTexture,
                    access: RenderGraphResourceAccessKind::Write,
                    attachment_ops: Some(RenderGraphAttachmentOps::load_store()),
                },
            ],
        );

        assert!(context.reads_texture("shadow-map"));
        assert!(context.reads_transient_texture("shadow-map"));
        assert!(!context.reads_texture("scene-color"));
    }

    #[test]
    fn metadata_context_keeps_external_reads_out_of_transient_texture_reads() {
        let context = RenderPassExecutionContext::with_graph_metadata_and_resources(
            "opaque-mesh",
            RenderPassExecutorId::new("mesh.opaque"),
            QueueLane::Graphics,
            Default::default(),
            vec![RenderGraphPassResourceAccess {
                name: "shadow-map".to_string(),
                kind: RenderGraphResourceKind::External,
                access: RenderGraphResourceAccessKind::Read,
                attachment_ops: None,
            }],
        );

        assert!(context.reads_texture("shadow-map"));
        assert!(!context.reads_transient_texture("shadow-map"));
    }
}
