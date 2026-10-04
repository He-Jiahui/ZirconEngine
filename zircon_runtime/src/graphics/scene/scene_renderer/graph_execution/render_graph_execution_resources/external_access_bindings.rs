use std::collections::HashMap;
use std::ops::Range;

use crate::render_graph::{
    CompiledRenderGraph, RenderGraphExternalResourceBinding, RenderGraphExternalResourceType,
    RenderGraphResourceAccessId, RenderGraphResourceAccessRange,
    RenderGraphTextureSubresourceRange, RenderGraphVersionedAccessKey,
};
use crate::rhi::TextureDesc;

use super::texture_views::{
    texture_range_covers_full_view, texture_subresource_view_descriptor,
    validate_texture_view_descriptor,
};
use super::RenderGraphExecutionResources;

#[derive(Debug)]
enum ExternalAccessBinding {
    Texture {
        key: RenderGraphVersionedAccessKey,
        view: wgpu::TextureView,
        desc: Option<TextureDesc>,
    },
    Buffer {
        key: RenderGraphVersionedAccessKey,
        buffer: wgpu::Buffer,
        range: Range<wgpu::BufferAddress>,
    },
}

impl ExternalAccessBinding {
    const fn key(&self) -> RenderGraphVersionedAccessKey {
        match self {
            Self::Texture { key, .. } | Self::Buffer { key, .. } => *key,
        }
    }
}

#[derive(Debug, Default)]
pub(super) struct RenderGraphExecutionExternalAccessBindings {
    bindings: HashMap<RenderGraphResourceAccessId, ExternalAccessBinding>,
    contracts_by_access: HashMap<
        RenderGraphResourceAccessId,
        (
            RenderGraphVersionedAccessKey,
            RenderGraphExternalResourceBinding,
        ),
    >,
}

impl RenderGraphExecutionExternalAccessBindings {
    pub(super) fn materialize(
        resources: &RenderGraphExecutionResources,
        graph: &CompiledRenderGraph,
    ) -> Result<Self, String> {
        let packet = graph.external_access_packet();
        let mut bindings = HashMap::with_capacity(packet.accesses().len());
        let mut contracts_by_access = HashMap::with_capacity(packet.accesses().len());
        let mut texture_views_by_scope: HashMap<_, wgpu::TextureView> = HashMap::new();
        for access in packet.accesses() {
            if contracts_by_access
                .insert(access.access_id, (access.key, access.binding))
                .is_some()
            {
                return Err(format!(
                    "external access packet contains duplicate logical contract {:?}",
                    access.access_id
                ));
            }
            let name = graph
                .resource_declaration(access.key.resource)
                .map(|declaration| declaration.name.as_str())
                .ok_or_else(|| {
                    format!(
                        "external access packet entry {:?} has no resource declaration",
                        access.access_id
                    )
                })?;
            let binding = match access.binding.resource_type {
                RenderGraphExternalResourceType::Texture => {
                    let Some(default_view) = resources.texture_view(name) else {
                        if access.binding.is_required() {
                            return Err(format!(
                                "required external texture `{name}` has no physical lease for access {:?}",
                                access.access_id
                            ));
                        }
                        continue;
                    };
                    let desc = resources.physical_texture_desc(name).cloned();
                    let view = match access.key.range {
                        RenderGraphResourceAccessRange::Texture(range) => {
                            let desc = desc.as_ref().ok_or_else(|| {
                                format!(
                                    "external texture `{name}` access {:?} has an exact scope but no physical texture descriptor",
                                    access.access_id
                                )
                            })?;
                            match texture_views_by_scope.get(&(access.key.resource, range)) {
                                Some(view) => view.clone(),
                                None => {
                                    let view = if let Some(texture) =
                                        resources.physical_texture(name)
                                    {
                                        let view_desc = texture_subresource_view_descriptor(range);
                                        validate_texture_view_descriptor(name, desc, &view_desc)?;
                                        texture.create_view(&view_desc)
                                    } else if texture_range_covers_full_view(range, desc) {
                                        default_view.clone()
                                    } else {
                                        return Err(format!(
                                            "external texture `{name}` access {:?} requires subresource scope {:?}, but its physical lease is view-only",
                                            access.access_id, range
                                        ));
                                    };
                                    texture_views_by_scope
                                        .insert((access.key.resource, range), view.clone());
                                    view
                                }
                            }
                        }
                        RenderGraphResourceAccessRange::UnresolvedExternal => default_view.clone(),
                        RenderGraphResourceAccessRange::Buffer(_) => {
                            return Err(format!(
                                "external texture `{name}` access {:?} has a buffer scope",
                                access.access_id
                            ));
                        }
                    };
                    ExternalAccessBinding::Texture {
                        key: access.key,
                        view,
                        desc,
                    }
                }
                RenderGraphExternalResourceType::Buffer => {
                    let Some(buffer) = resources.buffer(name).cloned() else {
                        if access.binding.is_required() {
                            return Err(format!(
                                "required external buffer `{name}` has no physical lease for access {:?}",
                                access.access_id
                            ));
                        }
                        continue;
                    };
                    let range = match access.key.range {
                        RenderGraphResourceAccessRange::Buffer(range) => {
                            let end = range
                                .size
                                .map(|size| range.offset.checked_add(size))
                                .unwrap_or(Some(buffer.size()))
                                .ok_or_else(|| {
                                    format!(
                                        "external buffer `{name}` access {:?} range overflows",
                                        access.access_id
                                    )
                                })?;
                            if range.offset >= end || end > buffer.size() {
                                return Err(format!(
                                    "external buffer `{name}` access {:?} range [{}..{}) exceeds physical buffer size {}",
                                    access.access_id,
                                    range.offset,
                                    end,
                                    buffer.size()
                                ));
                            }
                            range.offset..end
                        }
                        RenderGraphResourceAccessRange::UnresolvedExternal => 0..buffer.size(),
                        RenderGraphResourceAccessRange::Texture(_) => {
                            return Err(format!(
                                "external buffer `{name}` access {:?} has a texture scope",
                                access.access_id
                            ));
                        }
                    };
                    ExternalAccessBinding::Buffer {
                        key: access.key,
                        buffer,
                        range,
                    }
                }
                RenderGraphExternalResourceType::Unknown => continue,
            };
            if bindings.insert(access.access_id, binding).is_some() {
                return Err(format!(
                    "external access packet contains duplicate physical lease for access {:?}",
                    access.access_id
                ));
            }
        }
        Ok(Self {
            bindings,
            contracts_by_access,
        })
    }

    pub(super) fn texture_view(
        &self,
        access: RenderGraphResourceAccessId,
    ) -> Result<&wgpu::TextureView, String> {
        match self.bindings.get(&access) {
            Some(ExternalAccessBinding::Texture { view, .. }) => Ok(view),
            Some(ExternalAccessBinding::Buffer { .. }) => Err(format!(
                "external access {:?} is a buffer lease, not a texture view",
                access
            )),
            None => Err(format!(
                "external access {:?} has no materialized physical lease",
                access
            )),
        }
    }

    pub(super) fn optional_texture_view(
        &self,
        access: RenderGraphResourceAccessId,
    ) -> Result<Option<&wgpu::TextureView>, String> {
        match self.bindings.get(&access) {
            Some(ExternalAccessBinding::Texture { view, .. }) => Ok(Some(view)),
            Some(ExternalAccessBinding::Buffer { .. }) => Err(format!(
                "external access {:?} is a buffer lease, not a texture view",
                access
            )),
            None => Ok(None),
        }
    }

    pub(super) fn buffer_binding(
        &self,
        access: RenderGraphResourceAccessId,
    ) -> Result<(&wgpu::Buffer, Range<wgpu::BufferAddress>), String> {
        match self.bindings.get(&access) {
            Some(ExternalAccessBinding::Buffer { buffer, range, .. }) => {
                Ok((buffer, range.clone()))
            }
            Some(ExternalAccessBinding::Texture { .. }) => Err(format!(
                "external access {:?} is a texture lease, not a buffer binding",
                access
            )),
            None => Err(format!(
                "external access {:?} has no materialized physical lease",
                access
            )),
        }
    }

    pub(super) fn optional_buffer_binding(
        &self,
        access: RenderGraphResourceAccessId,
    ) -> Result<Option<(&wgpu::Buffer, Range<wgpu::BufferAddress>)>, String> {
        match self.bindings.get(&access) {
            Some(ExternalAccessBinding::Buffer { buffer, range, .. }) => {
                Ok(Some((buffer, range.clone())))
            }
            Some(ExternalAccessBinding::Texture { .. }) => Err(format!(
                "external access {:?} is a texture lease, not a buffer binding",
                access
            )),
            None => Ok(None),
        }
    }

    pub(super) fn texture_desc(
        &self,
        access: RenderGraphResourceAccessId,
    ) -> Result<TextureDesc, String> {
        match self.bindings.get(&access) {
            Some(ExternalAccessBinding::Texture {
                desc: Some(desc), ..
            }) => Ok(desc.clone()),
            Some(ExternalAccessBinding::Texture { desc: None, .. }) => Err(format!(
                "external access {:?} has no physical texture descriptor",
                access
            )),
            Some(ExternalAccessBinding::Buffer { .. }) => Err(format!(
                "external access {:?} is a buffer lease, not a texture descriptor",
                access
            )),
            None => Err(format!(
                "external access {:?} has no materialized physical lease",
                access
            )),
        }
    }

    pub(super) fn key(
        &self,
        access: RenderGraphResourceAccessId,
    ) -> Option<RenderGraphVersionedAccessKey> {
        self.bindings.get(&access).map(ExternalAccessBinding::key)
    }

    pub(super) fn contract(
        &self,
        access: RenderGraphResourceAccessId,
    ) -> Option<(
        RenderGraphVersionedAccessKey,
        RenderGraphExternalResourceBinding,
    )> {
        self.contracts_by_access.get(&access).copied()
    }
}

#[cfg(test)]
#[path = "tests/external_access_bindings.rs"]
mod tests;

impl RenderGraphExecutionResources {
    pub(in crate::graphics::scene::scene_renderer) fn external_access_contract_for_access(
        &self,
        access: RenderGraphResourceAccessId,
    ) -> Option<(
        RenderGraphVersionedAccessKey,
        RenderGraphExternalResourceBinding,
    )> {
        self.external_access_bindings.contract(access)
    }

    pub(in crate::graphics::scene::scene_renderer) fn external_access_key(
        &self,
        access: RenderGraphResourceAccessId,
    ) -> Option<RenderGraphVersionedAccessKey> {
        self.external_access_bindings.key(access)
    }

    pub(in crate::graphics::scene::scene_renderer) fn materialize_external_access_bindings(
        &mut self,
        graph: &CompiledRenderGraph,
    ) -> Result<(), String> {
        self.external_access_bindings =
            RenderGraphExecutionExternalAccessBindings::materialize(self, graph)?;
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn external_texture_view_for_access(
        &self,
        access: RenderGraphResourceAccessId,
    ) -> Result<&wgpu::TextureView, String> {
        self.external_access_bindings.texture_view(access)
    }

    pub(in crate::graphics::scene::scene_renderer) fn optional_external_texture_view_for_access(
        &self,
        access: RenderGraphResourceAccessId,
    ) -> Result<Option<&wgpu::TextureView>, String> {
        self.external_access_bindings.optional_texture_view(access)
    }

    pub(in crate::graphics::scene::scene_renderer) fn external_buffer_binding_for_access(
        &self,
        access: RenderGraphResourceAccessId,
    ) -> Result<(&wgpu::Buffer, Range<wgpu::BufferAddress>), String> {
        self.external_access_bindings.buffer_binding(access)
    }

    pub(in crate::graphics::scene::scene_renderer) fn optional_external_buffer_binding_for_access(
        &self,
        access: RenderGraphResourceAccessId,
    ) -> Result<Option<(&wgpu::Buffer, Range<wgpu::BufferAddress>)>, String> {
        self.external_access_bindings
            .optional_buffer_binding(access)
    }

    pub(in crate::graphics::scene::scene_renderer) fn external_texture_desc_for_access(
        &self,
        access: RenderGraphResourceAccessId,
    ) -> Result<TextureDesc, String> {
        self.external_access_bindings.texture_desc(access)
    }

    pub(in crate::graphics::scene::scene_renderer::graph_execution) fn clear_external_access_bindings(
        &mut self,
    ) {
        self.external_access_bindings = RenderGraphExecutionExternalAccessBindings::default();
    }
}
