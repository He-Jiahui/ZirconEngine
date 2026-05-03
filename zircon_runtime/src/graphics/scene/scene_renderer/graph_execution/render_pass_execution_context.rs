use crate::core::framework::render::{RenderFrameExtract, RenderPluginRendererOutputs};
use crate::core::math::UVec2;
use crate::graphics::types::ViewportRenderFrame;
use crate::render_graph::{PassFlags, QueueLane, RenderGraphPassResourceAccess, RenderPassId};

use super::{RenderGraphExecutionResources, RenderPassExecutorId};

#[derive(Debug)]
pub struct RenderPassGpuExecutionContext<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub encoder: &'a mut wgpu::CommandEncoder,
    frame: &'a ViewportRenderFrame,
    pub scene_bind_group: &'a wgpu::BindGroup,
    pub resources: &'a mut RenderGraphExecutionResources,
    pub plugin_outputs: &'a mut RenderPluginRendererOutputs,
}

impl<'a> RenderPassGpuExecutionContext<'a> {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::graphics::scene::scene_renderer) fn new(
        device: &'a wgpu::Device,
        queue: &'a wgpu::Queue,
        encoder: &'a mut wgpu::CommandEncoder,
        frame: &'a ViewportRenderFrame,
        scene_bind_group: &'a wgpu::BindGroup,
        resources: &'a mut RenderGraphExecutionResources,
        plugin_outputs: &'a mut RenderPluginRendererOutputs,
    ) -> Self {
        Self {
            device,
            queue,
            encoder,
            frame,
            scene_bind_group,
            resources,
            plugin_outputs,
        }
    }

    pub fn frame_extract(&self) -> &RenderFrameExtract {
        &self.frame.extract
    }

    pub fn viewport_size(&self) -> UVec2 {
        self.frame.viewport_size
    }
}

#[derive(Debug)]
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

impl RenderPassExecutionContext<'static> {
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
}

impl<'a> RenderPassExecutionContext<'a> {
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
}

#[cfg(test)]
mod tests {
    use super::RenderPassExecutionContext;
    use crate::graphics::RenderPassExecutorId;

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
}
