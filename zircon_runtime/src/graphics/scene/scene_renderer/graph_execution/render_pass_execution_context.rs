use crate::render_graph::{
    CompiledRenderGraph, PassFlags, QueueLane, RenderGraphAttachmentOps,
    RenderGraphPassResourceAccess, RenderGraphResource, RenderGraphResourceAccessKind,
    RenderGraphResourceDeclaration, RenderGraphResourceKind, RenderGraphResourceLifetime,
    RenderPassId,
};

use super::RenderPassExecutorId;

mod gpu;
mod resource_resolver;

pub use gpu::{ParticleGpuTransparentDrawContext, RenderPassGpuExecutionContext};
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
            .field("has_resource_resolver", &self.resource_resolver.is_some())
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

    pub fn with_gpu(mut self, gpu: RenderPassGpuExecutionContext<'a>) -> Self {
        self.gpu = Some(gpu.with_resource_resolver(self.resource_resolver));
        self
    }

    pub fn resource_resolver(&self) -> Option<RgResourceResolver<'a>> {
        self.resource_resolver
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
        let declaration = match self.resource_resolver {
            Some(resolver) => {
                Some(resolver.require_pass_resource_declaration_by_name(resource_name, access)?)
            }
            None => None,
        };
        let gpu = self.require_gpu()?;
        if let Some(declaration) = declaration {
            gpu.resources
                .require_texture_view_for_declaration(declaration)
        } else {
            gpu.resources.require_texture_view(resource_name)
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(in crate::graphics::scene::scene_renderer) fn require_buffer_by_name(
        &mut self,
        resource_name: &str,
        access: RenderGraphResourceAccessKind,
    ) -> Result<&wgpu::Buffer, String> {
        let declaration = match self.resource_resolver {
            Some(resolver) => {
                Some(resolver.require_pass_resource_declaration_by_name(resource_name, access)?)
            }
            None => None,
        };
        let gpu = self.require_gpu()?;
        if let Some(declaration) = declaration {
            gpu.resources.require_buffer_for_declaration(declaration)
        } else {
            gpu.resources.require_buffer(resource_name)
        }
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
        let graph_ops = self.graph_attachment_ops_for_write(resource_name)?;
        Some(
            self.gpu
                .as_ref()
                .map(|gpu| {
                    gpu.camera_stack_attachment_policy()
                        .apply_to_first_attachment_write(resource_name, graph_ops)
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

#[cfg(test)]
mod tests {
    use super::RenderPassExecutionContext;
    use crate::graphics::RenderPassExecutorId;
    use crate::render_graph::{
        QueueLane, RenderGraphAttachmentOps, RenderGraphBuilder, RenderGraphPassResourceAccess,
        RenderGraphResource, RenderGraphResourceAccessKind, RenderGraphResourceKind,
    };
    use crate::rhi::{TextureDesc, TextureFormat, TextureUsage};

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
                    name: "shadow-atlas".to_string(),
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

        assert!(context.reads_texture("shadow-atlas"));
        assert!(context.reads_transient_texture("shadow-atlas"));
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
                name: "shadow-atlas".to_string(),
                kind: RenderGraphResourceKind::External,
                access: RenderGraphResourceAccessKind::Read,
                attachment_ops: None,
            }],
        );

        assert!(context.reads_texture("shadow-atlas"));
        assert!(!context.reads_transient_texture("shadow-atlas"));
    }

    #[test]
    fn metadata_context_resolves_pass_resource_handles() {
        let mut builder = RenderGraphBuilder::new("resolver-context");
        let depth = builder.create_texture(TextureDesc::new(
            "scene-depth",
            32,
            32,
            TextureFormat::Depth32Float,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
        ));
        let color = builder.create_texture(TextureDesc::new(
            "scene-color",
            32,
            32,
            TextureFormat::Rgba8UnormSrgb,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
        ));
        let backbuffer = builder.import_external_resource("backbuffer");
        let depth_prepass = builder.add_pass("depth-prepass", QueueLane::Graphics);
        let opaque = builder.add_pass("opaque", QueueLane::Graphics);
        let present = builder.add_pass("present", QueueLane::Graphics);
        builder.write_texture(depth_prepass, depth).unwrap();
        builder.read_texture(opaque, depth).unwrap();
        builder.write_texture(opaque, color).unwrap();
        builder.read_texture(present, color).unwrap();
        builder.write_external(present, backbuffer).unwrap();

        let graph = builder.compile().unwrap();
        let pass = graph
            .passes()
            .iter()
            .find(|pass| pass.name == "opaque")
            .unwrap();
        let context =
            RenderPassExecutionContext::with_declared_graph_metadata_dependencies_and_resources(
                pass.name.clone(),
                RenderPassExecutorId::new(pass.executor_id.clone().unwrap_or_default()),
                pass.queue,
                pass.declared_queue,
                pass.flags,
                pass.dependencies.clone(),
                {
                    let mut resources = pass.resources.clone();
                    resources.push(RenderGraphPassResourceAccess {
                        name: "backbuffer".to_string(),
                        kind: RenderGraphResourceKind::External,
                        access: RenderGraphResourceAccessKind::Read,
                        attachment_ops: None,
                    });
                    resources
                },
            )
            .with_resource_resolver(&graph, pass.id);

        let depth_resource = RenderGraphResource::TransientTexture(depth);
        let color_resource = RenderGraphResource::TransientTexture(color);
        let backbuffer_resource = RenderGraphResource::External(backbuffer);

        assert_eq!(
            context
                .resource_declaration(depth_resource)
                .unwrap()
                .name
                .as_str(),
            "scene-depth"
        );
        assert_eq!(
            context
                .resource_lifetime(color_resource)
                .unwrap()
                .name
                .as_str(),
            "scene-color"
        );
        assert!(
            context.declares_resource_access(depth_resource, RenderGraphResourceAccessKind::Read)
        );
        assert_eq!(
            context
                .resource_resolver()
                .and_then(|resolver| resolver.pass_resource_declaration_by_name(
                    "scene-depth",
                    RenderGraphResourceAccessKind::Read
                ))
                .unwrap()
                .resource,
            depth_resource
        );
        assert!(context
            .resource_resolver()
            .and_then(|resolver| resolver.pass_resource_declaration_by_name(
                "scene-depth",
                RenderGraphResourceAccessKind::Write
            ))
            .is_none());
        assert!(
            context.declares_resource_access(color_resource, RenderGraphResourceAccessKind::Write)
        );
        assert!(!context
            .declares_resource_access(backbuffer_resource, RenderGraphResourceAccessKind::Write));
        assert!(
            !context.reads_texture("backbuffer"),
            "resolver-backed name queries must follow the compiled pass contract instead of stale context resource rows"
        );
    }

    #[test]
    fn resolver_backed_name_access_ignores_stale_context_resource_rows() {
        let mut builder = RenderGraphBuilder::new("resolver-name-access");
        let color = builder.create_texture(TextureDesc::new(
            "scene-color",
            32,
            32,
            TextureFormat::Rgba8UnormSrgb,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
        ));
        let output = builder.import_external_resource("viewport-output");
        let write = builder.add_pass("write", QueueLane::Graphics);
        let present = builder.add_pass("present", QueueLane::Graphics);
        builder.write_texture(write, color).unwrap();
        builder.read_texture(present, color).unwrap();
        builder.write_external(present, output).unwrap();
        let graph = builder.compile().unwrap();
        let pass = graph
            .passes()
            .iter()
            .find(|pass| pass.name == "write")
            .unwrap();
        let mut resources = pass.resources.clone();
        resources.push(RenderGraphPassResourceAccess {
            name: "viewport-output".to_string(),
            kind: RenderGraphResourceKind::External,
            access: RenderGraphResourceAccessKind::Read,
            attachment_ops: None,
        });
        let context =
            RenderPassExecutionContext::with_declared_graph_metadata_dependencies_and_resources(
                pass.name.clone(),
                RenderPassExecutorId::new(pass.executor_id.clone().unwrap_or_default()),
                pass.queue,
                pass.declared_queue,
                pass.flags,
                pass.dependencies.clone(),
                resources,
            )
            .with_resource_resolver(&graph, pass.id);

        assert!(!context
            .declares_resource_name_access("viewport-output", RenderGraphResourceAccessKind::Read));
        assert!(context
            .declares_resource_name_access("scene-color", RenderGraphResourceAccessKind::Write));
    }
}
