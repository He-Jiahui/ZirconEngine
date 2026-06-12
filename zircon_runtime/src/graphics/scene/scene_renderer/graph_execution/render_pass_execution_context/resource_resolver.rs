use crate::render_graph::{
    CompiledRenderGraph, CompiledRenderPass, RenderGraphPassResourceAccess, RenderGraphResource,
    RenderGraphResourceAccessKind, RenderGraphResourceDeclaration, RenderGraphResourceLifetime,
    RenderPassId,
};

#[derive(Clone, Copy)]
pub struct RenderPassResourceResolver<'a> {
    graph: &'a CompiledRenderGraph,
    pass_id: RenderPassId,
}

impl std::fmt::Debug for RenderPassResourceResolver<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RenderPassResourceResolver")
            .field("graph", &self.graph.name())
            .field("pass_id", &self.pass_id)
            .finish()
    }
}

impl<'a> RenderPassResourceResolver<'a> {
    pub fn new(graph: &'a CompiledRenderGraph, pass_id: RenderPassId) -> Self {
        Self { graph, pass_id }
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
        let Some(declaration) = self.resource_declaration(resource) else {
            return false;
        };
        self.pass_resources()
            .iter()
            .any(|access| access.name == declaration.name && access.kind == declaration.kind)
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
        let Some(declaration) = self.resource_declaration(resource) else {
            return None;
        };
        self.pass_resources().iter().find(|pass_access| {
            pass_access.name == declaration.name
                && pass_access.kind == declaration.kind
                && pass_access.access == access
        })
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

    pub fn pass_resources(&self) -> &'a [RenderGraphPassResourceAccess] {
        self.pass()
            .map(|pass| pass.resources.as_slice())
            .unwrap_or(&[])
    }

    fn pass(&self) -> Option<&'a CompiledRenderPass> {
        self.graph
            .passes()
            .iter()
            .find(|pass| pass.id == self.pass_id)
    }
}
