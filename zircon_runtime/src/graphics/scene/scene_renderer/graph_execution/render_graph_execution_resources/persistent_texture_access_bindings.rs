use std::collections::HashMap;

use crate::render_graph::{
    CompiledRenderGraph, RenderGraphResource, RenderGraphResourceAccessId,
    RenderGraphResourceAccessRange,
};

use super::RenderGraphExecutionResources;

#[derive(Debug, Default)]
pub(super) struct RenderGraphExecutionPersistentTextureAccessBindings {
    resources_by_access: HashMap<RenderGraphResourceAccessId, RenderGraphResource>,
    textures_by_resource: HashMap<RenderGraphResource, wgpu::Texture>,
    views_by_access: HashMap<RenderGraphResourceAccessId, wgpu::TextureView>,
}

impl RenderGraphExecutionPersistentTextureAccessBindings {
    pub(super) fn materialize(
        resources: &RenderGraphExecutionResources,
        graph: &CompiledRenderGraph,
    ) -> Result<Self, String> {
        let mut resources_by_access = HashMap::new();
        let mut textures_by_resource = HashMap::new();
        let mut views_by_access = HashMap::new();
        let mut views_by_scope = HashMap::new();

        for binding in graph.access_allocation_bindings() {
            if binding.physical_allocation.is_some() {
                continue;
            }
            let key = binding.key;
            let Some(declaration) = graph.resource_declaration(key.resource) else {
                return Err(format!(
                    "persistent texture access {:?} references an undeclared graph resource",
                    key.access_id
                ));
            };
            let Some(backing_resource) = graph.persistent_texture_backing_resource(key.resource)
            else {
                continue;
            };
            let backing_declaration = graph.resource_declaration(backing_resource).ok_or_else(|| {
                format!(
                    "persistent graph texture `{}` access {:?} references an undeclared backing resource",
                    declaration.name, key.access_id
                )
            })?;
            let RenderGraphResourceAccessRange::Texture(range) = key.range else {
                return Err(format!(
                    "persistent graph texture `{}` access {:?} has a non-texture scope",
                    declaration.name, key.access_id
                ));
            };
            let Some(texture) = resources.owned_texture(&backing_declaration.name) else {
                // Sparse or producer-owned persistent resources use a separate
                // typed lease path and must not be guessed from a logical name.
                continue;
            };
            textures_by_resource
                .entry(backing_resource)
                .or_insert_with(|| texture.clone());
            let view = match views_by_scope.get(&(backing_resource, range)) {
                Some(view) => view.clone(),
                None => {
                    let view = resources
                        .owned_texture_subresource_view(&backing_declaration.name, range)?;
                    views_by_scope.insert((backing_resource, range), view.clone());
                    view
                }
            };
            if resources_by_access
                .insert(key.access_id, backing_resource)
                .is_some()
            {
                return Err(format!(
                    "persistent graph texture access packet contains duplicate access {:?}",
                    key.access_id
                ));
            }
            if views_by_access.insert(key.access_id, view).is_some() {
                return Err(format!(
                    "persistent graph texture view packet contains duplicate access {:?}",
                    key.access_id
                ));
            }
        }

        Ok(Self {
            resources_by_access,
            textures_by_resource,
            views_by_access,
        })
    }

    pub(super) fn contains(&self, access: RenderGraphResourceAccessId) -> bool {
        self.resources_by_access.contains_key(&access)
    }

    pub(super) fn texture(
        &self,
        access: RenderGraphResourceAccessId,
    ) -> Result<&wgpu::Texture, String> {
        let resource = self.resources_by_access.get(&access).ok_or_else(|| {
            format!(
                "persistent graph texture access {:?} has no materialized physical lease",
                access
            )
        })?;
        self.textures_by_resource.get(resource).ok_or_else(|| {
            format!(
                "persistent graph texture access {:?} has no physical texture backing",
                access
            )
        })
    }

    pub(super) fn texture_view(
        &self,
        access: RenderGraphResourceAccessId,
    ) -> Result<&wgpu::TextureView, String> {
        self.views_by_access.get(&access).ok_or_else(|| {
            format!(
                "persistent graph texture access {:?} has no materialized texture view lease",
                access
            )
        })
    }

    #[cfg(test)]
    pub(super) fn binding_count(&self) -> usize {
        self.resources_by_access.len()
    }

    #[cfg(test)]
    pub(super) fn backing_count(&self) -> usize {
        self.textures_by_resource.len()
    }

    #[cfg(test)]
    pub(super) fn view_count(&self) -> usize {
        self.views_by_access.len()
    }
}

impl RenderGraphExecutionResources {
    pub(in crate::graphics::scene::scene_renderer::graph_execution) fn materialize_persistent_texture_access_bindings(
        &mut self,
        graph: &CompiledRenderGraph,
    ) -> Result<(), String> {
        self.persistent_texture_access_bindings =
            RenderGraphExecutionPersistentTextureAccessBindings::materialize(self, graph)?;
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer::graph_execution) fn clear_persistent_texture_access_bindings(
        &mut self,
    ) {
        self.persistent_texture_access_bindings =
            RenderGraphExecutionPersistentTextureAccessBindings::default();
    }

    pub(in crate::graphics::scene::scene_renderer) fn persistent_texture_for_access(
        &self,
        access: RenderGraphResourceAccessId,
    ) -> Result<&wgpu::Texture, String> {
        self.persistent_texture_access_bindings.texture(access)
    }

    pub(in crate::graphics::scene::scene_renderer) fn persistent_texture_view_for_access(
        &self,
        access: RenderGraphResourceAccessId,
    ) -> Result<&wgpu::TextureView, String> {
        self.persistent_texture_access_bindings.texture_view(access)
    }

    pub(in crate::graphics::scene::scene_renderer) fn graph_owned_texture_view_for_access(
        &self,
        access: RenderGraphResourceAccessId,
    ) -> Result<&wgpu::TextureView, String> {
        if self.access_bindings.contains(access) {
            return self.transient_texture_view_for_access(access);
        }
        if self.persistent_texture_access_bindings.contains(access) {
            return self.persistent_texture_view_for_access(access);
        }
        Err(format!(
            "graph-owned texture access {:?} has no transient or persistent texture view lease",
            access
        ))
    }

    pub(in crate::graphics::scene::scene_renderer) fn graph_owned_texture_for_access(
        &self,
        access: RenderGraphResourceAccessId,
    ) -> Result<&wgpu::Texture, String> {
        if self.access_bindings.contains(access) {
            return self.transient_texture_for_access(access);
        }
        if self.persistent_texture_access_bindings.contains(access) {
            return self.persistent_texture_for_access(access);
        }
        Err(format!(
            "graph-owned texture access {:?} has no transient or persistent physical lease",
            access
        ))
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer) fn persistent_texture_access_binding_count(
        &self,
    ) -> usize {
        self.persistent_texture_access_bindings.binding_count()
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer) fn persistent_texture_backing_count(
        &self,
    ) -> usize {
        self.persistent_texture_access_bindings.backing_count()
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer) fn persistent_texture_view_count(
        &self,
    ) -> usize {
        self.persistent_texture_access_bindings.view_count()
    }
}
