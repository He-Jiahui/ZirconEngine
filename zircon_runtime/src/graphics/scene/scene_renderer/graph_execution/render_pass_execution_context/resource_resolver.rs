use crate::render_graph::{
    CompiledRenderGraph, CompiledRenderPass, RenderGraphPassResourceAccess, RenderGraphResource,
    RenderGraphResourceAccessKind, RenderGraphResourceDeclaration, RenderGraphResourceLifetime,
    RenderPassId,
};

#[cfg(test)]
use super::super::RenderGraphExecutionResources;

#[derive(Clone, Copy)]
pub struct RgResourceResolver<'a> {
    graph: &'a CompiledRenderGraph,
    pass_id: RenderPassId,
    #[cfg(test)]
    physical: Option<&'a RenderGraphExecutionResources>,
}

impl std::fmt::Debug for RgResourceResolver<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RgResourceResolver")
            .field("graph", &self.graph.name())
            .field("pass_id", &self.pass_id)
            .field("has_physical", &self.has_physical_resources())
            .finish()
    }
}

impl<'a> RgResourceResolver<'a> {
    pub fn new(graph: &'a CompiledRenderGraph, pass_id: RenderPassId) -> Self {
        Self {
            graph,
            pass_id,
            #[cfg(test)]
            physical: None,
        }
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer) fn with_physical(
        graph: &'a CompiledRenderGraph,
        pass_id: RenderPassId,
        physical: &'a RenderGraphExecutionResources,
    ) -> Self {
        Self {
            graph,
            pass_id,
            physical: Some(physical),
        }
    }

    pub fn resource_declaration(
        &self,
        resource: RenderGraphResource,
    ) -> Option<&'a RenderGraphResourceDeclaration> {
        self.graph.resource_declaration(resource)
    }

    pub fn resource_declaration_by_name(
        &self,
        resource_name: &str,
    ) -> Option<&'a RenderGraphResourceDeclaration> {
        self.graph.resource_declaration_by_name(resource_name)
    }

    pub fn resource_lifetime(
        &self,
        resource: RenderGraphResource,
    ) -> Option<&'a RenderGraphResourceLifetime> {
        self.graph.resource_lifetime(resource)
    }

    pub fn resource_lifetime_by_name(
        &self,
        resource_name: &str,
    ) -> Option<&'a RenderGraphResourceLifetime> {
        self.graph.resource_lifetime_by_name(resource_name)
    }

    pub fn pass_declares_resource(&self, resource: RenderGraphResource) -> bool {
        self.graph
            .pass_resource_access(self.pass_id, resource, RenderGraphResourceAccessKind::Read)
            .is_some()
            || self
                .graph
                .pass_resource_access(self.pass_id, resource, RenderGraphResourceAccessKind::Write)
                .is_some()
    }

    pub fn pass_declares_resource_access(
        &self,
        resource: RenderGraphResource,
        access: RenderGraphResourceAccessKind,
    ) -> bool {
        self.pass_resource_access(resource, access).is_some()
    }

    pub fn pass_resource_access(
        &self,
        resource: RenderGraphResource,
        access: RenderGraphResourceAccessKind,
    ) -> Option<&'a RenderGraphPassResourceAccess> {
        self.graph
            .pass_resource_access(self.pass_id, resource, access)
    }

    pub fn pass_resource_access_by_name(
        &self,
        resource_name: &str,
        access: RenderGraphResourceAccessKind,
    ) -> Option<&'a RenderGraphPassResourceAccess> {
        let declaration = self.resource_declaration_by_name(resource_name)?;
        self.pass_resource_access(declaration.resource, access)
    }

    pub fn pass_resource_declaration_by_name(
        &self,
        resource_name: &str,
        access: RenderGraphResourceAccessKind,
    ) -> Option<&'a RenderGraphResourceDeclaration> {
        let declaration = self.resource_declaration_by_name(resource_name)?;
        self.pass_resource_access(declaration.resource, access)?;
        Some(declaration)
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer) fn texture_view_by_name(
        &self,
        resource_name: &str,
        access: RenderGraphResourceAccessKind,
    ) -> Result<&'a wgpu::TextureView, String> {
        let declaration = self.require_pass_resource_declaration_by_name(resource_name, access)?;
        self.texture_view(declaration.resource, access)
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer) fn texture_view(
        &self,
        resource: RenderGraphResource,
        access: RenderGraphResourceAccessKind,
    ) -> Result<&'a wgpu::TextureView, String> {
        let declaration = self.require_pass_resource_declaration(resource, access)?;
        self.physical_resources()?
            .require_texture_view_for_declaration(declaration)
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer) fn buffer_by_name(
        &self,
        resource_name: &str,
        access: RenderGraphResourceAccessKind,
    ) -> Result<&'a wgpu::Buffer, String> {
        let declaration = self.require_pass_resource_declaration_by_name(resource_name, access)?;
        self.buffer(declaration.resource, access)
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer) fn buffer(
        &self,
        resource: RenderGraphResource,
        access: RenderGraphResourceAccessKind,
    ) -> Result<&'a wgpu::Buffer, String> {
        let declaration = self.require_pass_resource_declaration(resource, access)?;
        self.physical_resources()?
            .require_buffer_for_declaration(declaration)
    }

    pub fn pass_resources(&self) -> &'a [RenderGraphPassResourceAccess] {
        self.pass()
            .map(|pass| pass.resources.as_slice())
            .unwrap_or(&[])
    }

    #[cfg(test)]
    fn require_pass_resource_declaration(
        &self,
        resource: RenderGraphResource,
        access: RenderGraphResourceAccessKind,
    ) -> Result<&'a RenderGraphResourceDeclaration, String> {
        let declaration = self.resource_declaration(resource).ok_or_else(|| {
            format!(
                "render graph pass `{}` references undeclared resource {:?}",
                self.pass_name(),
                resource
            )
        })?;
        if self.pass_resource_access(resource, access).is_none() {
            return Err(format!(
                "render graph pass `{}` did not declare {:?} access for resource `{}`",
                self.pass_name(),
                access,
                declaration.name
            ));
        }
        Ok(declaration)
    }

    pub(in crate::graphics::scene::scene_renderer) fn require_pass_resource_declaration_by_name(
        &self,
        resource_name: &str,
        access: RenderGraphResourceAccessKind,
    ) -> Result<&'a RenderGraphResourceDeclaration, String> {
        let declaration = self
            .resource_declaration_by_name(resource_name)
            .ok_or_else(|| {
                format!(
                    "render graph pass `{}` references undeclared resource `{resource_name}`",
                    self.pass_name()
                )
            })?;
        if self
            .pass_resource_access(declaration.resource, access)
            .is_none()
        {
            return Err(format!(
                "render graph pass `{}` did not declare {:?} access for resource `{}`",
                self.pass_name(),
                access,
                declaration.name
            ));
        }
        Ok(declaration)
    }

    #[cfg(test)]
    fn physical_resources(&self) -> Result<&'a RenderGraphExecutionResources, String> {
        self.physical.ok_or_else(|| {
            format!(
                "render graph pass `{}` has no physical execution resources attached to its resolver",
                self.pass_name()
            )
        })
    }

    fn pass_name(&self) -> &'a str {
        self.pass()
            .map(|pass| pass.name.as_str())
            .unwrap_or("<unknown>")
    }

    fn pass(&self) -> Option<&'a CompiledRenderPass> {
        self.graph.pass(self.pass_id)
    }

    fn has_physical_resources(&self) -> bool {
        #[cfg(test)]
        {
            self.physical.is_some()
        }
        #[cfg(not(test))]
        {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RgResourceResolver;
    use crate::graphics::backend::RenderBackend;
    use crate::graphics::scene::scene_renderer::graph_execution::{
        RenderGraphExecutionResources, TransientResourcePool,
    };
    use crate::render_graph::{
        PassFlags, QueueLane, RenderGraphBuilder, RenderGraphResourceAccessKind,
    };
    use crate::rhi::{TextureDesc, TextureFormat, TextureUsage};

    #[test]
    fn rg_resource_resolver_materialization_indices_follow_topologically_reordered_passes() {
        let mut builder = RenderGraphBuilder::new("resolver-compiled-indices");
        let color = builder.create_texture(TextureDesc::new(
            "scene-color",
            16,
            16,
            TextureFormat::Rgba8Unorm,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
        ));
        let consumer = builder.add_pass("consumer", QueueLane::Graphics);
        let producer = builder.add_pass("producer", QueueLane::Graphics);
        builder.write_texture(producer, color).unwrap();
        builder.read_texture(consumer, color).unwrap();
        builder.add_dependency(producer, consumer).unwrap();
        builder
            .set_pass_flags(
                consumer,
                PassFlags {
                    has_side_effects: true,
                    ..PassFlags::default()
                },
            )
            .unwrap();

        let graph = builder.compile().unwrap();
        assert_eq!(graph.passes()[0].id, producer);
        assert_eq!(graph.passes()[1].id, consumer);
        let color = graph
            .resource_declaration_by_name("scene-color")
            .expect("compiled graph retains scene-color")
            .resource;
        let resolver = RgResourceResolver::new(&graph, consumer);

        assert!(resolver.pass_declares_resource(color));
        assert!(resolver.pass_declares_resource_access(color, RenderGraphResourceAccessKind::Read));
        assert!(
            !resolver.pass_declares_resource_access(color, RenderGraphResourceAccessKind::Write)
        );
        assert_eq!(resolver.pass_resources().len(), 1);
        assert_eq!(resolver.pass_resources()[0].name, "scene-color");
    }

    #[test]
    fn rg_resource_resolver_requires_pass_declared_access_before_physical_texture_lookup() {
        let backend = RenderBackend::new_offscreen().unwrap();
        let mut builder = RenderGraphBuilder::new("resolver-physical-texture");
        let depth = builder.create_texture(TextureDesc::new(
            "scene-depth",
            16,
            16,
            TextureFormat::Depth32Float,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
        ));
        let color = builder.create_texture(TextureDesc::new(
            "scene-color",
            16,
            16,
            TextureFormat::Rgba8Unorm,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
        ));
        let output = builder.import_external_resource("viewport-output");
        let depth_prepass = builder.add_pass("depth-prepass", QueueLane::Graphics);
        let opaque = builder.add_pass("opaque", QueueLane::Graphics);
        let present = builder.add_pass("present", QueueLane::Graphics);
        builder.write_texture(depth_prepass, depth).unwrap();
        builder.read_texture(opaque, depth).unwrap();
        builder.write_texture(opaque, color).unwrap();
        builder.read_texture(present, color).unwrap();
        builder.write_external(present, output).unwrap();
        let graph = builder.compile().unwrap();
        let opaque_pass = graph
            .passes()
            .iter()
            .find(|pass| pass.name == "opaque")
            .unwrap();
        let mut resources = RenderGraphExecutionResources::new();
        let mut transient_pool = TransientResourcePool::default();
        transient_pool.begin_frame();
        resources
            .materialize_transient_resources_with_pool(&backend.device, &graph, &mut transient_pool)
            .unwrap();
        let resolver = RgResourceResolver::with_physical(&graph, opaque_pass.id, &resources);

        resolver
            .texture_view_by_name("scene-depth", RenderGraphResourceAccessKind::Read)
            .expect("declared depth read resolves through physical table");
        let error = resolver
            .texture_view_by_name("scene-color", RenderGraphResourceAccessKind::Read)
            .unwrap_err();

        assert!(
            error.contains("did not declare Read access for resource `scene-color`"),
            "{error}"
        );
    }
}
