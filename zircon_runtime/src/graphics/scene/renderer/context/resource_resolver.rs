use crate::render_graph::{
    CompiledRenderGraph, CompiledRenderPass, RenderGraphPassResourceAccess, RenderGraphResource,
    RenderGraphResourceAccessId, RenderGraphResourceAccessKind, RenderGraphResourceDeclaration,
    RenderGraphResourceKind, RenderGraphResourceLifetime, RenderPassId,
};

use super::super::RenderGraphExecutionResources;

#[derive(Clone, Copy)]
pub struct RgResourceResolver<'a> {
    graph: &'a CompiledRenderGraph,
    pass_id: RenderPassId,
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
            physical: None,
        }
    }

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

    pub(in crate::graphics::scene::scene_renderer) fn with_physical_resources(
        mut self,
        physical: &'a RenderGraphExecutionResources,
    ) -> Self {
        self.physical = Some(physical);
        self
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

    pub(in crate::graphics::scene::scene_renderer) fn texture_view_by_name(
        &self,
        resource_name: &str,
        access: RenderGraphResourceAccessKind,
    ) -> Result<&'a wgpu::TextureView, String> {
        let declaration = self.require_pass_resource_declaration_by_name(resource_name, access)?;
        self.texture_view(declaration.resource, access)
    }

    pub(in crate::graphics::scene::scene_renderer) fn texture_view(
        &self,
        resource: RenderGraphResource,
        access: RenderGraphResourceAccessKind,
    ) -> Result<&'a wgpu::TextureView, String> {
        let declaration = self.require_pass_resource_declaration(resource, access)?;
        let physical = self.physical_resources()?;
        if let Some(access_id) =
            self.exact_graph_owned_texture_access_by_name(&declaration.name, access)?
        {
            return physical.graph_owned_texture_view_for_access(access_id);
        }
        if let Some(access_id) = self.exact_external_access_by_name(&declaration.name, access)? {
            return physical.external_texture_view_for_access(access_id);
        }
        physical.require_texture_view_for_declaration(declaration)
    }

    pub(in crate::graphics::scene::scene_renderer) fn buffer_by_name(
        &self,
        resource_name: &str,
        access: RenderGraphResourceAccessKind,
    ) -> Result<&'a wgpu::Buffer, String> {
        let declaration = self.require_pass_resource_declaration_by_name(resource_name, access)?;
        self.buffer(declaration.resource, access)
    }

    pub(in crate::graphics::scene::scene_renderer) fn buffer(
        &self,
        resource: RenderGraphResource,
        access: RenderGraphResourceAccessKind,
    ) -> Result<&'a wgpu::Buffer, String> {
        let declaration = self.require_pass_resource_declaration(resource, access)?;
        let physical = self.physical_resources()?;
        if let Some(access_id) = self.exact_transient_access_by_name(&declaration.name, access)? {
            let (buffer, _) = physical.transient_buffer_binding_for_access(access_id)?;
            return Ok(buffer);
        }
        if let Some(access_id) = self.exact_external_access_by_name(&declaration.name, access)? {
            let (buffer, _) = physical.external_buffer_binding_for_access(access_id)?;
            return Ok(buffer);
        }
        physical.require_buffer_for_declaration(declaration)
    }

    pub fn pass_resources(&self) -> &'a [RenderGraphPassResourceAccess] {
        self.pass()
            .map(|pass| pass.resources.as_slice())
            .unwrap_or(&[])
    }

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

    pub(in crate::graphics::scene::scene_renderer) fn has_physical_resources(&self) -> bool {
        self.physical.is_some()
    }

    /// Returns the exact transient access identity when this pass has one.
    ///
    /// The compiled `(pass, resource, access)` index is the hot-path lookup;
    /// the pass-row scan below runs only to distinguish an absent declaration
    /// from an ambiguous malformed one.
    ///
    /// Persistent transient resources and external resources deliberately do
    /// not enter the frame transient binding table. They return `None` so the
    /// caller can use their separately validated physical lease path. An
    /// ambiguous transient name is an error instead of silently selecting one
    /// of multiple same-kind accesses.
    pub(in crate::graphics::scene::scene_renderer) fn exact_transient_access_by_name(
        &self,
        resource_name: &str,
        access: RenderGraphResourceAccessKind,
    ) -> Result<Option<RenderGraphResourceAccessId>, String> {
        let Some(declaration) = self.resource_declaration_by_name(resource_name) else {
            return Ok(None);
        };
        if !matches!(
            declaration.kind,
            RenderGraphResourceKind::TransientTexture | RenderGraphResourceKind::TransientBuffer
        ) {
            return Ok(None);
        }
        let Some(access_id) = self
            .graph
            .access_id_for(self.pass_id, declaration.resource, access)
        else {
            let pass_declares_access = self.pass().is_some_and(|pass| {
                pass.resources
                    .iter()
                    .any(|row| row.name == resource_name && row.access == access)
            });
            if !pass_declares_access {
                // Leave the normal declaration error (including its access kind)
                // to the typed resolver path below.
                return Ok(None);
            }
            // A resource without a compiler physical allocation is either
            // persistent or otherwise outside the transient table. Only that
            // compatibility case may continue through the declaration path.
            if self
                .graph
                .physical_allocation_id_for_resource(declaration.resource)
                .is_none()
            {
                return Ok(None);
            }
            return Err(format!(
                "render graph pass `{}` has an ambiguous transient {:?} access for resource `{}`; exact access identity is required",
                self.pass_name(),
                access,
                resource_name
            ));
        };
        if self
            .graph
            .physical_allocation_id_for_access(access_id)
            .is_none()
        {
            return Ok(None);
        }
        Ok(Some(access_id))
    }

    /// Returns the exact access identity for an allocation-backed or persistent graph texture.
    ///
    /// Persistent textures deliberately sit outside transient alias slots, but
    /// still have compiler access identities and frame-materialized WGPU view
    /// leases. Sparse/provider-owned textures fail later at the lease table
    /// instead of falling back to a logical resource name.
    pub(in crate::graphics::scene::scene_renderer) fn exact_graph_owned_texture_access_by_name(
        &self,
        resource_name: &str,
        access: RenderGraphResourceAccessKind,
    ) -> Result<Option<RenderGraphResourceAccessId>, String> {
        let Some(declaration) = self.resource_declaration_by_name(resource_name) else {
            return Ok(None);
        };
        if declaration.kind != RenderGraphResourceKind::TransientTexture {
            return Ok(None);
        }
        let Some(access_id) = self
            .graph
            .access_id_for(self.pass_id, declaration.resource, access)
        else {
            let pass_declares_access = self.pass().is_some_and(|pass| {
                pass.resources
                    .iter()
                    .any(|row| row.name == resource_name && row.access == access)
            });
            if !pass_declares_access {
                return Ok(None);
            }
            return Err(format!(
                "render graph pass `{}` has an ambiguous graph-owned texture {:?} access for resource `{}`; exact access identity is required",
                self.pass_name(),
                access,
                resource_name
            ));
        };
        let allocation_backed = self
            .graph
            .physical_allocation_id_for_access(access_id)
            .is_some();
        let persistent = self
            .graph
            .persistent_texture_backing_resource(declaration.resource)
            .is_some();
        Ok((allocation_backed || persistent).then_some(access_id))
    }

    /// Returns the exact external access identity when this pass declares one.
    ///
    /// The compiled `(pass, resource, access)` index is consulted before the
    /// diagnostic ambiguity scan, keeping steady-frame resolution O(1).
    ///
    /// Unknown report-only imports deliberately return `None` so legacy name
    /// lookup remains available. Typed external imports must resolve through
    /// the frame lease table, and ambiguous same-kind rows fail closed.
    pub(in crate::graphics::scene::scene_renderer) fn exact_external_access_by_name(
        &self,
        resource_name: &str,
        access: RenderGraphResourceAccessKind,
    ) -> Result<Option<RenderGraphResourceAccessId>, String> {
        let Some(declaration) = self.resource_declaration_by_name(resource_name) else {
            return Ok(None);
        };
        if declaration.kind != RenderGraphResourceKind::External {
            return Ok(None);
        }
        // Unknown report-only imports intentionally retain the legacy name
        // lookup. They have no physical type contract and are not included in
        // the typed external lease table.
        if declaration.external_binding.resource_type
            == crate::render_graph::RenderGraphExternalResourceType::Unknown
        {
            return Ok(None);
        }
        let Some(access_id) = self
            .graph
            .access_id_for(self.pass_id, declaration.resource, access)
        else {
            let pass_declares_access = self.pass().is_some_and(|pass| {
                pass.resources
                    .iter()
                    .any(|row| row.name == resource_name && row.access == access)
            });
            if !pass_declares_access {
                return Ok(None);
            }
            return Err(format!(
                "render graph pass `{}` has an ambiguous external {:?} access for resource `{}`; exact access identity is required",
                self.pass_name(),
                access,
                resource_name
            ));
        };
        Ok(Some(access_id))
    }
}

#[cfg(test)]
#[path = "tests/resource_resolver.rs"]
mod tests;
