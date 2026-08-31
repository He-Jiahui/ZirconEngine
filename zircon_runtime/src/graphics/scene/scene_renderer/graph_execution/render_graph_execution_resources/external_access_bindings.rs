use std::collections::HashMap;
use std::ops::Range;

use crate::render_graph::{
    CompiledRenderGraph, RenderGraphExternalResourceType, RenderGraphResourceAccessId,
    RenderGraphResourceAccessRange, RenderGraphTextureAspect, RenderGraphTextureSubresourceRange,
};
use crate::rhi::TextureDesc;

use super::RenderGraphExecutionResources;
use super::texture_views::{texture_subresource_view_descriptor, validate_texture_view_descriptor};

#[derive(Debug)]
enum ExternalAccessBinding {
    Texture {
        view: wgpu::TextureView,
        desc: Option<TextureDesc>,
    },
    Buffer {
        buffer: wgpu::Buffer,
        range: Range<wgpu::BufferAddress>,
    },
}

#[derive(Debug, Default)]
pub(super) struct RenderGraphExecutionExternalAccessBindings {
    bindings: HashMap<RenderGraphResourceAccessId, ExternalAccessBinding>,
}

impl RenderGraphExecutionExternalAccessBindings {
    pub(super) fn materialize(
        resources: &RenderGraphExecutionResources,
        graph: &CompiledRenderGraph,
    ) -> Result<Self, String> {
        let packet = graph.external_access_packet();
        let mut bindings = HashMap::with_capacity(packet.accesses().len());
        let mut texture_views_by_scope = HashMap::new();
        for access in packet.accesses() {
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
                    ExternalAccessBinding::Texture { view, desc }
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
                    ExternalAccessBinding::Buffer { buffer, range }
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
        Ok(Self { bindings })
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
            Some(ExternalAccessBinding::Buffer { buffer, range }) => Ok((buffer, range.clone())),
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
            Some(ExternalAccessBinding::Buffer { buffer, range }) => {
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
}

fn texture_range_covers_full_view(
    range: RenderGraphTextureSubresourceRange,
    desc: &TextureDesc,
) -> bool {
    let covers_mips = range.base_mip_level == 0 && range.mip_level_count == Some(desc.mip_levels);
    let array_layers = match desc.dimension {
        crate::rhi::TextureDimension::D2Array | crate::rhi::TextureDimension::Cube => desc.depth,
        crate::rhi::TextureDimension::D1
        | crate::rhi::TextureDimension::D2
        | crate::rhi::TextureDimension::D3 => 1,
    };
    let covers_layers =
        range.base_array_layer == 0 && range.array_layer_count == Some(array_layers);
    let covers_aspects = match range.aspect {
        RenderGraphTextureAspect::All => true,
        RenderGraphTextureAspect::Color => !desc.format.is_depth(),
        RenderGraphTextureAspect::Depth => desc.format.is_depth() && !desc.format.has_stencil(),
        RenderGraphTextureAspect::Stencil => false,
    };
    covers_mips && covers_layers && covers_aspects
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphics::backend::RenderBackend;
    use crate::render_graph::{
        PassFlags, QueueLane, RenderGraphBufferRange, RenderGraphBuilder,
        RenderGraphExternalResourceBinding, RenderGraphResourceAccessIntent,
        RenderGraphResourceAccessKind, RenderGraphResourceAccessRange, RenderGraphShaderStages,
        RenderGraphTextureSubresourceRange,
    };
    use crate::rhi::{BufferDesc, BufferUsage, TextureDesc, TextureFormat, TextureUsage};

    #[test]
    fn required_external_lease_is_fail_closed_when_physical_buffer_is_missing() {
        let mut builder = RenderGraphBuilder::new("external-lease-missing");
        let buffer = builder.import_present_external_buffer_with_binding(
            "external-buffer",
            BufferDesc::new("external-buffer", 256, BufferUsage::STORAGE),
            RenderGraphExternalResourceBinding::required_buffer(),
        );
        let pass = builder.add_pass("external-writer", QueueLane::AsyncCompute);
        builder.write_storage_external(pass, buffer).unwrap();
        builder
            .set_pass_flags(
                pass,
                PassFlags {
                    has_side_effects: true,
                    ..PassFlags::default()
                },
            )
            .unwrap();
        let graph = builder.compile().unwrap();
        let resources = RenderGraphExecutionResources::new();
        let error = RenderGraphExecutionExternalAccessBindings::materialize(&resources, &graph)
            .expect_err("required external leases must be present before encoding");
        assert!(error.contains("required external buffer `external-buffer`"));
    }

    #[test]
    fn external_lease_table_is_keyed_by_compiled_access_identity() {
        let Ok(backend) = RenderBackend::new_offscreen() else {
            return;
        };
        let mut builder = RenderGraphBuilder::new("external-lease-access-id");
        let buffer = builder.import_present_external_buffer_with_binding(
            "external-buffer",
            BufferDesc::new("external-buffer", 256, BufferUsage::STORAGE),
            RenderGraphExternalResourceBinding::required_buffer(),
        );
        let pass = builder.add_pass("external-writer", QueueLane::AsyncCompute);
        builder.write_storage_external(pass, buffer).unwrap();
        builder
            .set_pass_flags(
                pass,
                PassFlags {
                    has_side_effects: true,
                    ..PassFlags::default()
                },
            )
            .unwrap();
        let graph = builder.compile().unwrap();
        let native = backend.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("external-lease-access-id-buffer"),
            size: 256,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let mut resources = RenderGraphExecutionResources::new();
        resources.insert_buffer("external-buffer", native);
        resources
            .materialize_external_access_bindings(&graph)
            .expect("typed external lease should materialize");
        let access_id = graph.access_id_at(pass, 0).unwrap();
        let (_, range) = resources
            .external_buffer_binding_for_access(access_id)
            .expect("compiled access identity should resolve its lease");
        assert_eq!(range, 0..256);
    }

    #[test]
    fn external_lease_table_preserves_concrete_buffer_access_window() {
        let Ok(backend) = RenderBackend::new_offscreen() else {
            return;
        };
        let mut builder = RenderGraphBuilder::new("external-lease-buffer-window");
        let buffer = builder.import_present_external_buffer_with_binding(
            "external-buffer",
            BufferDesc::new("external-buffer", 256, BufferUsage::STORAGE),
            RenderGraphExternalResourceBinding::required_buffer(),
        );
        let pass = builder.add_pass("external-writer", QueueLane::AsyncCompute);
        builder
            .access_external(
                pass,
                buffer,
                RenderGraphResourceAccessKind::Write,
                RenderGraphResourceAccessRange::Buffer(RenderGraphBufferRange::new(32, Some(64))),
                RenderGraphResourceAccessIntent::storage_buffer_read_write(
                    RenderGraphShaderStages::COMPUTE,
                ),
                None,
            )
            .unwrap();
        builder
            .set_pass_flags(
                pass,
                PassFlags {
                    has_side_effects: true,
                    ..PassFlags::default()
                },
            )
            .unwrap();
        let graph = builder.compile().unwrap();
        let native = backend.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("external-lease-buffer-window"),
            size: 256,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let mut resources = RenderGraphExecutionResources::new();
        resources.insert_buffer("external-buffer", native);
        resources
            .materialize_external_access_bindings(&graph)
            .unwrap();
        let access_id = graph.access_id_at(pass, 0).unwrap();
        let (_, range) = resources
            .external_buffer_binding_for_access(access_id)
            .unwrap();
        assert_eq!(range, 32..96);
    }

    #[test]
    fn external_texture_lease_materializes_exact_scope_from_physical_backing() {
        let Ok(backend) = RenderBackend::new_offscreen() else {
            return;
        };
        let (graph, pass, desc) = exact_external_texture_graph("exact-external-texture");
        let texture = create_external_texture(&backend, "exact-external-texture", &desc);
        let default_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut resources = RenderGraphExecutionResources::new();
        resources.import_borrowed_texture("external-texture", &texture, &default_view, desc);

        resources
            .materialize_external_access_bindings(&graph)
            .expect("physical texture backing must materialize its exact mip lease");

        let access_id = graph.access_id_at(pass, 0).unwrap();
        assert!(
            resources
                .external_texture_view_for_access(access_id)
                .is_ok()
        );
    }

    #[test]
    fn exact_external_texture_scope_rejects_a_view_only_physical_lease() {
        let Ok(backend) = RenderBackend::new_offscreen() else {
            return;
        };
        let (graph, _pass, desc) = exact_external_texture_graph("view-only-external-texture");
        let texture = create_external_texture(&backend, "view-only-external-texture", &desc);
        let default_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut resources = RenderGraphExecutionResources::new();
        resources.import_borrowed_texture_view_with_physical_desc(
            "external-texture",
            &default_view,
            desc,
        );

        let error = resources
            .materialize_external_access_bindings(&graph)
            .expect_err("a default view cannot represent a partial mip lease");

        assert!(error.contains("requires subresource scope"));
        assert!(error.contains("physical lease is view-only"));
    }

    #[test]
    fn exact_full_external_texture_scope_accepts_a_view_only_physical_lease() {
        let Ok(backend) = RenderBackend::new_offscreen() else {
            return;
        };
        let desc = external_texture_desc("full-view-only-external-texture");
        let mut builder = RenderGraphBuilder::new("full-view-only-external-texture");
        let texture = builder.import_present_external_texture_with_binding(
            "external-texture",
            desc.clone(),
            RenderGraphExternalResourceBinding::required_texture(),
        );
        let pass = builder.add_pass("external-reader", QueueLane::AsyncCompute);
        builder
            .read_external_with_access(
                pass,
                texture,
                RenderGraphResourceAccessRange::Texture(RenderGraphTextureSubresourceRange::full()),
                RenderGraphResourceAccessIntent::sampled_texture(RenderGraphShaderStages::COMPUTE),
            )
            .unwrap();
        builder.set_pass_flags(pass, non_cullable()).unwrap();
        let graph = builder.compile().unwrap();
        let native = create_external_texture(&backend, "full-view-only-external-texture", &desc);
        let default_view = native.create_view(&wgpu::TextureViewDescriptor::default());
        let mut resources = RenderGraphExecutionResources::new();
        resources.import_borrowed_texture_view_with_physical_desc(
            "external-texture",
            &default_view,
            desc,
        );

        resources
            .materialize_external_access_bindings(&graph)
            .expect("a full-scope access may reuse its producer-supplied default view");
        let access_id = graph.access_id_at(pass, 0).unwrap();
        assert!(
            resources
                .external_texture_view_for_access(access_id)
                .is_ok()
        );
    }

    fn exact_external_texture_graph(
        name: &'static str,
    ) -> (
        CompiledRenderGraph,
        crate::render_graph::RenderPassId,
        TextureDesc,
    ) {
        let desc = external_texture_desc(name);
        let mut builder = RenderGraphBuilder::new(name);
        let texture = builder.import_present_external_texture_with_binding(
            "external-texture",
            desc.clone(),
            RenderGraphExternalResourceBinding::required_texture(),
        );
        let pass = builder.add_pass("external-reader", QueueLane::AsyncCompute);
        builder
            .read_external_with_access(
                pass,
                texture,
                RenderGraphResourceAccessRange::Texture(
                    RenderGraphTextureSubresourceRange::single_mip(2),
                ),
                RenderGraphResourceAccessIntent::sampled_texture(RenderGraphShaderStages::COMPUTE),
            )
            .unwrap();
        builder.set_pass_flags(pass, non_cullable()).unwrap();
        (builder.compile().unwrap(), pass, desc)
    }

    fn external_texture_desc(name: &'static str) -> TextureDesc {
        TextureDesc::new(
            name,
            32,
            16,
            TextureFormat::Rgba16Float,
            TextureUsage::SAMPLED,
        )
        .with_mip_levels(4)
    }

    fn create_external_texture(
        backend: &RenderBackend,
        label: &'static str,
        desc: &TextureDesc,
    ) -> wgpu::Texture {
        backend.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: desc.width,
                height: desc.height,
                depth_or_array_layers: desc.depth,
            },
            mip_level_count: desc.mip_levels,
            sample_count: desc.sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        })
    }

    fn non_cullable() -> PassFlags {
        PassFlags {
            has_side_effects: true,
            ..PassFlags::default()
        }
    }
}

impl RenderGraphExecutionResources {
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
