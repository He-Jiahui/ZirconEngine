use crate::core::math::UVec2;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::types::ViewportRenderRegion;
use crate::render_graph::{
    CompiledRenderGraph, CompiledRenderGraphComputeBindingAccessPacket,
    CompiledRenderGraphComputeDispatchAccessPacket, PassFlags, QueueLane,
    RenderGraphAttachmentLoadOp, RenderGraphAttachmentOps, RenderGraphComputePassMetadata,
    RenderGraphComputeWorkload, RenderGraphPassResourceAccess, RenderGraphResource,
    RenderGraphResourceAccessId, RenderGraphResourceAccessKind, RenderGraphResourceDeclaration,
    RenderGraphResourceKind, RenderGraphResourceLifetime, RenderPassId,
};

use super::RenderPassExecutorId;

mod gpu;
mod resource_resolver;

pub use gpu::{
    ParticleGpuTransparentDrawContext, RenderPassBufferUploadRecorder, RenderPassBufferUploadSink,
    RenderPassGpuExecutionContext, RenderPassGpuNativeContext, RenderPassGpuRecordingContext,
    RenderPassGpuResourceFactory,
};
pub(in crate::graphics::scene::scene_renderer) use gpu::{
    RenderPassMeshCommandLists, RenderPassPostProcessStackContext,
};
pub use resource_resolver::RgResourceResolver;

pub struct RenderPassExecutionContext<'a> {
    pub pass_name: String,
    pub executor_id: RenderPassExecutorId,
    pub declared_queue: QueueLane,
    pub queue: QueueLane,
    pub flags: PassFlags,
    pub dependencies: Vec<RenderPassId>,
    pub resources: Vec<RenderGraphPassResourceAccess>,
    compiled_access_ids: Option<&'a [RenderGraphResourceAccessId]>,
    compute_workload: Option<&'a RenderGraphComputeWorkload>,
    compute_pass_metadata: Option<&'a RenderGraphComputePassMetadata>,
    compute_binding_access_packet: Option<&'a CompiledRenderGraphComputeBindingAccessPacket>,
    compute_dispatch_access_packet: Option<&'a CompiledRenderGraphComputeDispatchAccessPacket>,
    resource_streamer: Option<&'a ResourceStreamer>,
    resource_resolver: Option<RgResourceResolver<'a>>,
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
            .field(
                "has_compiled_access_ids",
                &self.compiled_access_ids.is_some(),
            )
            .field("has_compute_workload", &self.compute_workload.is_some())
            .field(
                "has_compute_pass_metadata",
                &self.compute_pass_metadata.is_some(),
            )
            .field(
                "has_compute_binding_access_packet",
                &self.compute_binding_access_packet.is_some(),
            )
            .field(
                "has_compute_dispatch_access_packet",
                &self.compute_dispatch_access_packet.is_some(),
            )
            .field("has_resource_streamer", &self.resource_streamer.is_some())
            .field("has_resource_resolver", &self.resource_resolver.is_some())
            .field("has_gpu", &self.gpu.is_some())
            .finish()
    }
}

impl<'a> RenderPassExecutionContext<'a> {
    pub fn new(pass_name: impl Into<String>, executor_id: RenderPassExecutorId) -> Self {
        Self::with_graph_metadata(
            pass_name,
            executor_id,
            QueueLane::Graphics,
            PassFlags::default(),
        )
    }

    pub fn with_graph_metadata(
        pass_name: impl Into<String>,
        executor_id: RenderPassExecutorId,
        queue: QueueLane,
        flags: PassFlags,
    ) -> Self {
        Self::with_graph_metadata_and_resources(pass_name, executor_id, queue, flags, Vec::new())
    }

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
            compiled_access_ids: None,
            compute_workload: None,
            compute_pass_metadata: None,
            compute_binding_access_packet: None,
            compute_dispatch_access_packet: None,
            resource_streamer: None,
            resource_resolver: None,
            gpu: None,
        }
    }

    pub fn with_resource_resolver(
        mut self,
        graph: &'a CompiledRenderGraph,
        pass_id: RenderPassId,
    ) -> Self {
        self.resource_resolver = Some(RgResourceResolver::new(graph, pass_id));
        self
    }

    /// Attaches the exact immutable access identities selected by the compiled packet.
    ///
    /// Product contexts must keep this one-to-one with their declared access rows;
    /// an executor must never infer an identity from a resource label.
    pub(in crate::graphics) fn with_compiled_access_ids(
        mut self,
        pass_id: RenderPassId,
        access_ids: &'a [RenderGraphResourceAccessId],
    ) -> Result<Self, String> {
        if access_ids.len() != self.resources.len() {
            return Err(format!(
                "render pass `{}` has {} declared resource access row(s) but {} compiled access identity value(s)",
                self.pass_name,
                self.resources.len(),
                access_ids.len()
            ));
        }
        for (access_ordinal, access_id) in access_ids.iter().copied().enumerate() {
            if access_id.pass() != pass_id {
                return Err(format!(
                    "render pass `{}` compiled access identity {:?} at ordinal {access_ordinal} belongs to pass {:?}, expected {:?}",
                    self.pass_name,
                    access_id,
                    access_id.pass(),
                    pass_id
                ));
            }
            if access_id.access_index() != access_ordinal {
                return Err(format!(
                    "render pass `{}` compiled access identity {:?} has access ordinal {}, expected {access_ordinal}",
                    self.pass_name,
                    access_id,
                    access_id.access_index(),
                ));
            }
        }
        self.compiled_access_ids = Some(access_ids);
        Ok(self)
    }

    pub fn with_compute_pass_metadata(
        mut self,
        compute_pass_metadata: Option<&'a RenderGraphComputePassMetadata>,
    ) -> Self {
        self.compute_pass_metadata = compute_pass_metadata;
        self
    }

    /// Attaches compiler-owned binding access identities for a live generic-compute pass.
    /// Physical external leases remain a separate packet and are intentionally absent here.
    pub(in crate::graphics) fn with_compute_binding_access_packet(
        mut self,
        compute_binding_access_packet: Option<&'a CompiledRenderGraphComputeBindingAccessPacket>,
    ) -> Self {
        self.compute_binding_access_packet = compute_binding_access_packet;
        self
    }

    /// Attaches the compiler-selected physical target for an indirect or per-pixel dispatch.
    pub(in crate::graphics) fn with_compute_dispatch_access_packet(
        mut self,
        compute_dispatch_access_packet: Option<&'a CompiledRenderGraphComputeDispatchAccessPacket>,
    ) -> Self {
        self.compute_dispatch_access_packet = compute_dispatch_access_packet;
        self
    }

    pub fn with_compute_workload(
        mut self,
        compute_workload: Option<&'a RenderGraphComputeWorkload>,
    ) -> Self {
        self.compute_workload = compute_workload;
        self
    }

    pub(in crate::graphics::scene::scene_renderer) fn with_resource_streamer(
        mut self,
        resource_streamer: Option<&'a ResourceStreamer>,
    ) -> Self {
        self.resource_streamer = resource_streamer;
        self
    }

    pub fn with_gpu(mut self, gpu: RenderPassGpuExecutionContext<'a>) -> Self {
        self.resource_resolver = self
            .resource_resolver
            .map(|resolver| resolver.with_physical_resources(gpu.resources));
        self.gpu = Some(gpu.with_resource_resolver(self.resource_resolver));
        self
    }

    pub fn resource_resolver(&self) -> Option<RgResourceResolver<'a>> {
        self.resource_resolver
    }

    /// Returns the exact compiled identities for this pass when it originated
    /// from a `RenderGraphExecutionPacket`.
    pub fn compiled_access_ids(&self) -> Option<&'a [RenderGraphResourceAccessId]> {
        self.compiled_access_ids
    }

    pub fn compute_pass_metadata(&self) -> Option<&'a RenderGraphComputePassMetadata> {
        self.compute_pass_metadata
    }

    pub fn compute_workload(&self) -> Option<&'a RenderGraphComputeWorkload> {
        self.compute_workload
    }

    pub fn compute_binding_access_packet(
        &self,
    ) -> Option<&'a CompiledRenderGraphComputeBindingAccessPacket> {
        self.compute_binding_access_packet
    }

    pub fn compute_dispatch_access_packet(
        &self,
    ) -> Option<&'a CompiledRenderGraphComputeDispatchAccessPacket> {
        self.compute_dispatch_access_packet
    }

    pub(in crate::graphics::scene::scene_renderer) fn resource_streamer(
        &self,
    ) -> Option<&'a ResourceStreamer> {
        self.resource_streamer
    }

    pub fn resource_declaration(
        &self,
        resource: RenderGraphResource,
    ) -> Option<&'a RenderGraphResourceDeclaration> {
        self.resource_resolver
            .and_then(|resolver| resolver.resource_declaration(resource))
    }

    pub fn resource_lifetime(
        &self,
        resource: RenderGraphResource,
    ) -> Option<&'a RenderGraphResourceLifetime> {
        self.resource_resolver
            .and_then(|resolver| resolver.resource_lifetime(resource))
    }

    pub fn declares_resource_access(
        &self,
        resource: RenderGraphResource,
        access: RenderGraphResourceAccessKind,
    ) -> bool {
        self.resource_resolver
            .is_some_and(|resolver| resolver.pass_declares_resource_access(resource, access))
    }

    pub fn declares_resource_name_access(
        &self,
        resource_name: &str,
        access: RenderGraphResourceAccessKind,
    ) -> bool {
        if let Some(resolver) = self.resource_resolver {
            return resolver
                .pass_resource_access_by_name(resource_name, access)
                .is_some();
        }
        self.resources
            .iter()
            .any(|resource| resource.name == resource_name && resource.access == access)
    }

    pub(in crate::graphics::scene::scene_renderer) fn require_texture_view_by_name(
        &mut self,
        resource_name: &str,
        access: RenderGraphResourceAccessKind,
    ) -> Result<&wgpu::TextureView, String> {
        if let Some(resolver) = self
            .resource_resolver
            .filter(RgResourceResolver::has_physical_resources)
        {
            return resolver.texture_view_by_name(resource_name, access);
        }
        self.require_gpu()?
            .resources
            .require_texture_view(resource_name)
    }

    pub(in crate::graphics::scene::scene_renderer) fn require_buffer_by_name(
        &mut self,
        resource_name: &str,
        access: RenderGraphResourceAccessKind,
    ) -> Result<&wgpu::Buffer, String> {
        if let Some(resolver) = self
            .resource_resolver
            .filter(RgResourceResolver::has_physical_resources)
        {
            return resolver.buffer_by_name(resource_name, access);
        }
        self.require_gpu()?.resources.require_buffer(resource_name)
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

    pub fn uses_queue_fallback(&self) -> bool {
        self.declared_queue != self.queue
    }

    pub fn attachment_ops_for_write(
        &self,
        resource_name: &str,
    ) -> Option<RenderGraphAttachmentOps> {
        let graph_ops = self.graph_attachment_ops_for_write(resource_name)?;
        Some(
            self.gpu
                .as_ref()
                .map(|gpu| {
                    let attachment_ops = gpu
                        .camera_stack_attachment_policy()
                        .apply_to_first_attachment_write(resource_name, graph_ops);
                    preserve_physical_output_attachment_ops_for_partitioned_viewport(
                        resource_name,
                        attachment_ops,
                        gpu.render_region_for_write_resource(resource_name),
                        gpu.viewport_size(),
                    )
                })
                .unwrap_or(graph_ops),
        )
    }

    fn graph_attachment_ops_for_write(
        &self,
        resource_name: &str,
    ) -> Option<RenderGraphAttachmentOps> {
        if let Some(resolver) = self.resource_resolver {
            return resolver
                .pass_resource_access_by_name(resource_name, RenderGraphResourceAccessKind::Write)
                .filter(|resource| {
                    matches!(
                        resource.kind,
                        RenderGraphResourceKind::TransientTexture
                            | RenderGraphResourceKind::External
                    )
                })
                .and_then(|resource| resource.attachment_ops);
        }
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
        if let Some(resolver) = self.resource_resolver {
            return resolver
                .pass_resource_access_by_name(resource_name, RenderGraphResourceAccessKind::Read)
                .is_some_and(|access| {
                    matches!(
                        access.kind,
                        RenderGraphResourceKind::TransientTexture
                            | RenderGraphResourceKind::External
                    )
                });
        }
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
        if let Some(resolver) = self.resource_resolver {
            return resolver
                .pass_resource_access_by_name(resource_name, RenderGraphResourceAccessKind::Read)
                .is_some_and(|access| access.kind == RenderGraphResourceKind::TransientTexture);
        }
        self.resources.iter().any(|resource| {
            resource.name == resource_name
                && resource.access == RenderGraphResourceAccessKind::Read
                && resource.kind == RenderGraphResourceKind::TransientTexture
        })
    }
}

fn preserve_physical_output_attachment_ops_for_partitioned_viewport(
    resource_name: &str,
    attachment_ops: RenderGraphAttachmentOps,
    render_region: ViewportRenderRegion,
    target_size: UVec2,
) -> RenderGraphAttachmentOps {
    if attachment_ops.load != RenderGraphAttachmentLoadOp::Clear
        || !gpu::writes_physical_output_resource(resource_name)
        || render_region_covers_target(render_region, target_size)
    {
        return attachment_ops;
    }
    RenderGraphAttachmentOps {
        load: RenderGraphAttachmentLoadOp::Load,
        store: attachment_ops.store,
    }
}

fn render_region_covers_target(render_region: ViewportRenderRegion, target_size: UVec2) -> bool {
    let target_size = UVec2::new(target_size.x.max(1), target_size.y.max(1));
    render_region.physical_position() == UVec2::ZERO && render_region.physical_size() == target_size
}

#[cfg(test)]
#[path = "render_pass_execution_context_tests.rs"]
mod tests;
