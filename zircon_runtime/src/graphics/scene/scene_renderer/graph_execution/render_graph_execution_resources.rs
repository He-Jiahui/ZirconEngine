use std::collections::BTreeMap;

use crate::core::framework::render::{
    RenderGraphExecutionAliasRecord, RenderGraphExecutionAliasReport,
    RenderGraphExecutionResourceReport, RenderGraphMaterializationReport,
};
use crate::render_graph::{
    CompiledRenderGraph, RenderGraphResourceDeclaration, RenderGraphResourceDesc,
    RenderGraphResourceKind,
};
use crate::rhi::{BufferDesc, TextureDesc, TextureDimension};

use super::TransientResourcePool;

#[derive(Clone, Copy, Debug)]
pub(in crate::graphics::scene::scene_renderer) struct RenderGraphImportedFinalTarget<'a> {
    pub view: &'a wgpu::TextureView,
}

#[derive(Default, Debug)]
pub struct RenderGraphExecutionResources {
    imported_texture_views: BTreeMap<String, wgpu::TextureView>,
    imported_textures: BTreeMap<String, wgpu::Texture>,
    imported_texture_descs: BTreeMap<String, TextureDesc>,
    owned_textures: BTreeMap<String, wgpu::Texture>,
    owned_texture_descs: BTreeMap<String, TextureDesc>,
    owned_texture_backings: BTreeMap<String, String>,
    buffers: BTreeMap<String, wgpu::Buffer>,
    owned_buffer_descs: BTreeMap<String, BufferDesc>,
    buffer_backings: BTreeMap<String, String>,
}

impl RenderGraphExecutionResources {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn import_texture_view(
        &mut self,
        name: impl Into<String>,
        view: wgpu::TextureView,
    ) -> Option<wgpu::TextureView> {
        self.imported_texture_views.insert(name.into(), view)
    }

    pub(in crate::graphics::scene::scene_renderer) fn import_borrowed_texture_view(
        &mut self,
        name: impl Into<String>,
        view: &wgpu::TextureView,
    ) -> Option<wgpu::TextureView> {
        self.import_texture_view(name, view.clone())
    }

    pub(in crate::graphics::scene::scene_renderer) fn import_borrowed_texture(
        &mut self,
        name: impl Into<String>,
        texture: &wgpu::Texture,
        view: &wgpu::TextureView,
        desc: TextureDesc,
    ) -> Option<wgpu::TextureView> {
        let name = name.into();
        self.imported_textures.insert(name.clone(), texture.clone());
        self.imported_texture_descs.insert(name.clone(), desc);
        self.import_borrowed_texture_view(name, view)
    }

    pub fn insert_buffer(
        &mut self,
        name: impl Into<String>,
        buffer: wgpu::Buffer,
    ) -> Option<wgpu::Buffer> {
        let name = name.into();
        self.buffer_backings.insert(name.clone(), name.clone());
        self.buffers.insert(name, buffer)
    }

    pub(in crate::graphics::scene::scene_renderer) fn bind_execution_owned_buffer(
        &mut self,
        logical_name: impl Into<String>,
        backing_name: impl Into<String>,
        buffer: &wgpu::Buffer,
    ) -> Option<String> {
        let logical_name = logical_name.into();
        let backing_name = backing_name.into();
        self.buffers.insert(backing_name.clone(), buffer.clone());
        self.buffer_backings.insert(logical_name, backing_name)
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer) fn materialize_transient_resources(
        &mut self,
        device: &wgpu::Device,
        graph: &CompiledRenderGraph,
    ) -> Result<(), String> {
        super::materialization::materialize_transient_resources(self, device, graph, None)
    }

    pub(in crate::graphics::scene::scene_renderer) fn materialize_transient_resources_with_pool(
        &mut self,
        device: &wgpu::Device,
        graph: &CompiledRenderGraph,
        pool: &mut TransientResourcePool,
    ) -> Result<(), String> {
        super::materialization::materialize_transient_resources(self, device, graph, Some(pool))
    }

    pub(in crate::graphics::scene::scene_renderer) fn release_transient_backings_into_pool(
        &mut self,
        pool: &mut TransientResourcePool,
    ) {
        self.imported_texture_views.clear();
        self.imported_textures.clear();
        self.imported_texture_descs.clear();
        self.owned_texture_backings.clear();
        self.buffer_backings.clear();

        for (backing_name, texture) in std::mem::take(&mut self.owned_textures) {
            if let Some(desc) = self.owned_texture_descs.remove(&backing_name) {
                pool.release_texture(desc, texture);
            }
        }
        self.owned_texture_descs.clear();

        for (backing_name, buffer) in std::mem::take(&mut self.buffers) {
            if let Some(desc) = self.owned_buffer_descs.remove(&backing_name) {
                pool.release_buffer(desc, buffer);
            }
        }
        self.owned_buffer_descs.clear();
    }

    pub(in crate::graphics::scene::scene_renderer::graph_execution) fn texture_view(
        &self,
        name: &str,
    ) -> Option<&wgpu::TextureView> {
        self.imported_texture_views.get(name)
    }

    pub(in crate::graphics::scene::scene_renderer) fn buffer(
        &self,
        name: &str,
    ) -> Option<&wgpu::Buffer> {
        self.buffer_backing(name)
            .and_then(|backing| self.buffers.get(backing))
    }

    pub(in crate::graphics::scene::scene_renderer) fn owned_texture(
        &self,
        name: &str,
    ) -> Option<&wgpu::Texture> {
        self.owned_texture_backing(name)
            .and_then(|backing| self.owned_textures.get(backing))
    }

    pub(in crate::graphics::scene::scene_renderer) fn physical_texture(
        &self,
        name: &str,
    ) -> Option<&wgpu::Texture> {
        self.owned_texture(name)
            .or_else(|| self.imported_textures.get(name))
    }

    pub(in crate::graphics::scene::scene_renderer) fn owned_texture_desc(
        &self,
        name: &str,
    ) -> Option<&TextureDesc> {
        self.owned_texture_backing(name)
            .and_then(|backing| self.owned_texture_descs.get(backing))
    }

    pub(in crate::graphics::scene::scene_renderer) fn physical_texture_desc(
        &self,
        name: &str,
    ) -> Option<&TextureDesc> {
        self.owned_texture_desc(name)
            .or_else(|| self.imported_texture_descs.get(name))
    }

    pub(in crate::graphics::scene::scene_renderer::graph_execution) fn require_owned_texture_desc(
        &self,
        name: &str,
    ) -> Result<&TextureDesc, String> {
        self.owned_texture_desc(name).ok_or_else(|| {
            format!(
                "render graph execution texture resource `{name}` is not an owned transient texture"
            )
        })
    }

    pub(in crate::graphics::scene::scene_renderer) fn owned_texture_mip_view(
        &self,
        name: &str,
        mip_level: u32,
    ) -> Result<wgpu::TextureView, String> {
        let backing = self.owned_texture_backing(name).ok_or_else(|| {
            format!(
                "render graph execution texture resource `{name}` is not an owned transient texture"
            )
        })?;
        let texture = self.owned_textures.get(backing).ok_or_else(|| {
            format!(
                "render graph execution texture resource `{name}` backing `{backing}` is missing"
            )
        })?;
        let desc = self.owned_texture_descs.get(backing).ok_or_else(|| {
            format!("render graph execution texture resource `{name}` is missing its descriptor")
        })?;
        if mip_level >= desc.mip_levels {
            return Err(format!(
                "render graph execution texture resource `{name}` mip level {mip_level} is outside mip_levels {}",
                desc.mip_levels
            ));
        }
        Ok(texture.create_view(&texture_mip_view_descriptor(mip_level)))
    }

    pub(in crate::graphics::scene::scene_renderer) fn owned_texture_full_mip_view(
        &self,
        name: &str,
    ) -> Result<wgpu::TextureView, String> {
        let backing = self.owned_texture_backing(name).ok_or_else(|| {
            format!(
                "render graph execution texture resource `{name}` is not an owned transient texture"
            )
        })?;
        let texture = self.owned_textures.get(backing).ok_or_else(|| {
            format!(
                "render graph execution texture resource `{name}` backing `{backing}` is missing"
            )
        })?;
        let desc = self.owned_texture_descs.get(backing).ok_or_else(|| {
            format!("render graph execution texture resource `{name}` is missing its descriptor")
        })?;
        Ok(texture.create_view(&texture_full_mip_view_descriptor(desc.mip_levels)))
    }

    pub(in crate::graphics::scene::scene_renderer) fn owned_texture_view_with_descriptor(
        &self,
        name: &str,
        descriptor: &wgpu::TextureViewDescriptor<'_>,
    ) -> Result<wgpu::TextureView, String> {
        let backing = self.owned_texture_backing(name).ok_or_else(|| {
            format!(
                "render graph execution texture resource `{name}` is not an owned transient texture"
            )
        })?;
        let texture = self.owned_textures.get(backing).ok_or_else(|| {
            format!(
                "render graph execution texture resource `{name}` backing `{backing}` is missing"
            )
        })?;
        let desc = self.owned_texture_descs.get(backing).ok_or_else(|| {
            format!("render graph execution texture resource `{name}` is missing its descriptor")
        })?;
        validate_owned_texture_view_descriptor(name, desc, descriptor)?;
        Ok(texture.create_view(descriptor))
    }

    pub(in crate::graphics::scene::scene_renderer) fn owned_texture_mip_level_count(
        &self,
        name: &str,
    ) -> Option<u32> {
        self.owned_texture_descs
            .get(self.owned_texture_backing(name).unwrap_or(name))
            .map(|desc| desc.mip_levels)
    }

    pub(in crate::graphics::scene::scene_renderer::graph_execution) fn require_texture_view(
        &self,
        name: &str,
    ) -> Result<&wgpu::TextureView, String> {
        self.texture_view(name)
            .ok_or_else(|| format!("render graph execution texture resource `{name}` is not bound"))
    }

    pub(in crate::graphics::scene::scene_renderer::graph_execution) fn require_texture_view_for_declaration(
        &self,
        declaration: &RenderGraphResourceDeclaration,
    ) -> Result<&wgpu::TextureView, String> {
        if declaration.kind == RenderGraphResourceKind::TransientBuffer {
            return Err(format!(
                "render graph execution resource `{}` is a buffer declaration, not a texture view",
                declaration.name
            ));
        }
        self.require_texture_view(&declaration.name)
    }

    pub(in crate::graphics::scene::scene_renderer::graph_execution) fn require_texture_desc_for_declaration(
        &self,
        declaration: &RenderGraphResourceDeclaration,
    ) -> Result<TextureDesc, String> {
        if declaration.kind == RenderGraphResourceKind::TransientBuffer {
            return Err(format!(
                "render graph execution resource `{}` is a buffer declaration, not a texture descriptor",
                declaration.name
            ));
        }
        match &declaration.desc {
            RenderGraphResourceDesc::Texture(desc) => Ok(desc.clone()),
            RenderGraphResourceDesc::External => {
                self.physical_texture_desc(&declaration.name)
                    .cloned()
                    .ok_or_else(|| {
                        format!(
                            "render graph execution external texture resource `{}` is missing its physical descriptor",
                            declaration.name
                        )
                    })
            }
            RenderGraphResourceDesc::Buffer(_) => Err(format!(
                "render graph execution resource `{}` is a buffer declaration, not a texture descriptor",
                declaration.name
            )),
        }
    }

    pub(in crate::graphics::scene::scene_renderer::graph_execution) fn require_buffer(
        &self,
        name: &str,
    ) -> Result<&wgpu::Buffer, String> {
        self.buffer(name)
            .ok_or_else(|| format!("render graph execution buffer resource `{name}` is not bound"))
    }

    pub(in crate::graphics::scene::scene_renderer::graph_execution) fn require_buffer_for_declaration(
        &self,
        declaration: &RenderGraphResourceDeclaration,
    ) -> Result<&wgpu::Buffer, String> {
        if declaration.kind == RenderGraphResourceKind::TransientTexture {
            return Err(format!(
                "render graph execution resource `{}` is a texture declaration, not a buffer",
                declaration.name
            ));
        }
        self.require_buffer(&declaration.name)
    }

    pub fn has_texture_view(&self, name: &str) -> bool {
        self.imported_texture_views.contains_key(name)
    }

    pub fn has_buffer(&self, name: &str) -> bool {
        self.buffer_backing(name)
            .is_some_and(|backing| self.buffers.contains_key(backing))
    }

    pub fn has_bound_resource(&self, name: &str) -> bool {
        self.has_texture_view(name) || self.has_buffer(name)
    }

    pub(super) fn bound_texture_view_names(&self) -> impl Iterator<Item = &str> {
        self.imported_texture_views.keys().map(String::as_str)
    }

    pub(super) fn bound_buffer_names(&self) -> impl Iterator<Item = &str> {
        self.buffer_backings.keys().map(String::as_str)
    }

    pub fn resource_report(&self) -> RenderGraphExecutionResourceReport {
        let texture_view_count = self.imported_texture_views.len();
        let owned_texture_count = self.owned_textures.len();
        let owned_backed_texture_view_count = self
            .imported_texture_views
            .keys()
            .filter(|name| self.is_owned_backed_texture_view(name))
            .count();
        RenderGraphExecutionResourceReport::new(
            texture_view_count,
            texture_view_count.saturating_sub(owned_backed_texture_view_count),
            owned_texture_count,
            self.buffers.len(),
        )
    }

    pub fn validate_materialized_graph_resources(
        &self,
        graph: &CompiledRenderGraph,
    ) -> Result<RenderGraphMaterializationReport, String> {
        super::materialization_validation::validate_materialized_graph_resources(self, graph)
    }

    pub fn resource_alias_report(&self) -> RenderGraphExecutionAliasReport {
        let mut texture_aliases = self
            .owned_texture_backings
            .iter()
            .map(|(logical_name, backing_name)| {
                RenderGraphExecutionAliasRecord::new(logical_name.clone(), backing_name.clone())
            })
            .collect::<Vec<_>>();
        for logical_name in self.imported_texture_views.keys() {
            if self.owned_texture_backings.contains_key(logical_name) {
                continue;
            }
            let Some((parent, mip_level)) =
                super::transient_materialization::ssr_pyramid_mip_alias(logical_name)
            else {
                continue;
            };
            if self.owned_texture(parent).is_some() {
                texture_aliases.push(RenderGraphExecutionAliasRecord::new(
                    logical_name.clone(),
                    format!("{parent}:mip{mip_level}"),
                ));
            }
        }
        texture_aliases.sort_by(|left, right| {
            left.logical_name
                .cmp(&right.logical_name)
                .then_with(|| left.backing_name.cmp(&right.backing_name))
        });

        let mut buffer_aliases = self
            .buffer_backings
            .iter()
            .filter(|(logical_name, backing_name)| {
                self.owned_buffer_descs.contains_key(*backing_name)
                    || logical_name.as_str() != backing_name.as_str()
            })
            .map(|(logical_name, backing_name)| {
                RenderGraphExecutionAliasRecord::new(logical_name.clone(), backing_name.clone())
            })
            .collect::<Vec<_>>();
        buffer_aliases.sort_by(|left, right| {
            left.logical_name
                .cmp(&right.logical_name)
                .then_with(|| left.backing_name.cmp(&right.backing_name))
        });

        RenderGraphExecutionAliasReport::new(texture_aliases, buffer_aliases)
    }

    fn is_owned_backed_texture_view(&self, name: &str) -> bool {
        self.owned_texture_backings.contains_key(name)
            || super::transient_materialization::ssr_pyramid_mip_alias(name)
                .is_some_and(|(parent, _)| self.owned_texture(parent).is_some())
    }

    pub fn import_texture_alias(
        &mut self,
        alias: impl Into<String>,
        source: &wgpu::Texture,
    ) -> Option<wgpu::TextureView> {
        self.import_texture_view(
            alias,
            source.create_view(&wgpu::TextureViewDescriptor::default()),
        )
    }

    pub(super) fn insert_owned_texture(
        &mut self,
        name: impl Into<String>,
        texture: wgpu::Texture,
        desc: TextureDesc,
    ) -> Option<wgpu::Texture> {
        let name = name.into();
        let view = texture.create_view(&texture_mip_view_descriptor(0));
        self.import_texture_view(name.clone(), view);
        self.owned_texture_backings
            .insert(name.clone(), name.clone());
        self.owned_texture_descs.insert(name.clone(), desc);
        self.owned_textures.insert(name, texture)
    }

    pub(super) fn insert_owned_texture_backing(
        &mut self,
        backing_name: impl Into<String>,
        texture: wgpu::Texture,
        desc: TextureDesc,
    ) -> Option<wgpu::Texture> {
        let backing_name = backing_name.into();
        self.owned_texture_descs.insert(backing_name.clone(), desc);
        self.owned_textures.insert(backing_name, texture)
    }

    pub(super) fn bind_owned_texture_view(
        &mut self,
        logical_name: impl Into<String>,
        backing_name: &str,
    ) -> Result<Option<wgpu::TextureView>, String> {
        let logical_name = logical_name.into();
        let view = self
            .owned_textures
            .get(backing_name)
            .ok_or_else(|| {
                format!("render graph execution texture backing `{backing_name}` is missing")
            })?
            .create_view(&texture_mip_view_descriptor(0));
        self.owned_texture_backings
            .insert(logical_name.clone(), backing_name.to_string());
        Ok(self.import_texture_view(logical_name, view))
    }

    fn owned_texture_backing(&self, name: &str) -> Option<&str> {
        self.owned_texture_backings.get(name).map(String::as_str)
    }

    pub(super) fn insert_buffer_backing(
        &mut self,
        backing_name: impl Into<String>,
        buffer: wgpu::Buffer,
        desc: BufferDesc,
    ) -> Option<wgpu::Buffer> {
        let backing_name = backing_name.into();
        self.owned_buffer_descs.insert(backing_name.clone(), desc);
        self.buffers.insert(backing_name, buffer)
    }

    pub(super) fn bind_buffer(
        &mut self,
        logical_name: impl Into<String>,
        backing_name: &str,
    ) -> Option<String> {
        self.buffer_backings
            .insert(logical_name.into(), backing_name.to_string())
    }

    fn buffer_backing<'a>(&'a self, name: &'a str) -> Option<&'a str> {
        if let Some(backing) = self.buffer_backings.get(name) {
            return Some(backing);
        }
        self.buffers.contains_key(name).then_some(name)
    }
}

fn texture_mip_view_descriptor(mip_level: u32) -> wgpu::TextureViewDescriptor<'static> {
    wgpu::TextureViewDescriptor {
        base_mip_level: mip_level,
        mip_level_count: Some(1),
        ..Default::default()
    }
}

fn texture_full_mip_view_descriptor(mip_level_count: u32) -> wgpu::TextureViewDescriptor<'static> {
    wgpu::TextureViewDescriptor {
        base_mip_level: 0,
        mip_level_count: Some(mip_level_count),
        ..Default::default()
    }
}

fn validate_owned_texture_view_descriptor(
    name: &str,
    texture_desc: &TextureDesc,
    view_desc: &wgpu::TextureViewDescriptor<'_>,
) -> Result<(), String> {
    validate_owned_texture_view_format(name, texture_desc, view_desc)?;
    validate_owned_texture_view_usage(name, texture_desc, view_desc)?;
    validate_owned_texture_view_dimension(name, texture_desc, view_desc)?;
    validate_owned_texture_view_mip_range(name, texture_desc, view_desc)?;
    validate_owned_texture_view_array_range(name, texture_desc, view_desc)?;
    Ok(())
}

fn validate_owned_texture_view_format(
    name: &str,
    texture_desc: &TextureDesc,
    view_desc: &wgpu::TextureViewDescriptor<'_>,
) -> Result<(), String> {
    let expected_format = super::materialization::wgpu_texture_format(texture_desc.format);
    if view_desc
        .format
        .is_some_and(|view_format| view_format != expected_format)
    {
        return Err(format!(
            "render graph execution texture resource `{name}` view format {:?} does not match texture format {:?}",
            view_desc.format.unwrap(),
            expected_format
        ));
    }
    Ok(())
}

fn validate_owned_texture_view_usage(
    name: &str,
    texture_desc: &TextureDesc,
    view_desc: &wgpu::TextureViewDescriptor<'_>,
) -> Result<(), String> {
    let Some(requested_usage) = view_desc.usage else {
        return Ok(());
    };
    let texture_usages =
        super::materialization::wgpu_texture_usages(texture_desc.format, texture_desc.usage);
    if !texture_usages.contains(requested_usage) {
        return Err(format!(
            "render graph execution texture resource `{name}` view usage {:?} is not allowed by texture usages {:?}",
            requested_usage, texture_usages
        ));
    }
    Ok(())
}

fn validate_owned_texture_view_dimension(
    name: &str,
    texture_desc: &TextureDesc,
    view_desc: &wgpu::TextureViewDescriptor<'_>,
) -> Result<(), String> {
    let Some(view_dimension) = view_desc.dimension else {
        return Ok(());
    };
    if texture_view_dimension_allowed(texture_desc.dimension, view_dimension) {
        return Ok(());
    }
    Err(format!(
        "render graph execution texture resource `{name}` view dimension {:?} is not compatible with texture dimension {:?}",
        view_dimension, texture_desc.dimension
    ))
}

fn texture_view_dimension_allowed(
    texture_dimension: TextureDimension,
    view_dimension: wgpu::TextureViewDimension,
) -> bool {
    match texture_dimension {
        TextureDimension::D1 => matches!(view_dimension, wgpu::TextureViewDimension::D1),
        TextureDimension::D2 => matches!(view_dimension, wgpu::TextureViewDimension::D2),
        TextureDimension::D2Array => matches!(
            view_dimension,
            wgpu::TextureViewDimension::D2 | wgpu::TextureViewDimension::D2Array
        ),
        TextureDimension::D3 => matches!(view_dimension, wgpu::TextureViewDimension::D3),
        TextureDimension::Cube => matches!(
            view_dimension,
            wgpu::TextureViewDimension::D2
                | wgpu::TextureViewDimension::D2Array
                | wgpu::TextureViewDimension::Cube
                | wgpu::TextureViewDimension::CubeArray
        ),
    }
}

fn validate_owned_texture_view_mip_range(
    name: &str,
    texture_desc: &TextureDesc,
    view_desc: &wgpu::TextureViewDescriptor<'_>,
) -> Result<(), String> {
    let base = view_desc.base_mip_level;
    let count = view_desc
        .mip_level_count
        .unwrap_or_else(|| texture_desc.mip_levels.saturating_sub(base));
    if count == 0 || base.saturating_add(count) > texture_desc.mip_levels {
        return Err(format!(
            "render graph execution texture resource `{name}` view mip range [{base}..{}) is outside mip_levels {}",
            base.saturating_add(count),
            texture_desc.mip_levels
        ));
    }
    Ok(())
}

fn validate_owned_texture_view_array_range(
    name: &str,
    texture_desc: &TextureDesc,
    view_desc: &wgpu::TextureViewDescriptor<'_>,
) -> Result<(), String> {
    let base = view_desc.base_array_layer;
    let count = view_desc
        .array_layer_count
        .unwrap_or_else(|| texture_desc.depth.saturating_sub(base));
    if count == 0 || base.saturating_add(count) > texture_desc.depth {
        return Err(format!(
            "render graph execution texture resource `{name}` view array range [{base}..{}) is outside depth/array_layers {}",
            base.saturating_add(count),
            texture_desc.depth
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::RenderGraphExecutionResources;
    use crate::render_graph::{
        PassFlags, QueueLane, RenderGraphBuilder, RenderGraphResource, RenderGraphResourceKind,
    };
    use crate::rhi::{BufferDesc, BufferUsage, TextureDesc, TextureFormat, TextureUsage};

    #[test]
    fn resource_registry_reports_missing_named_resources() {
        let resources = RenderGraphExecutionResources::new();

        assert_eq!(
            resources.require_texture_view("scene-color").unwrap_err(),
            "render graph execution texture resource `scene-color` is not bound"
        );
        assert_eq!(
            resources
                .require_buffer("particles.gpu.alive-indices")
                .unwrap_err(),
            "render graph execution buffer resource `particles.gpu.alive-indices` is not bound"
        );
    }

    #[test]
    fn resource_registry_validates_declaration_kind_before_name_lookup() {
        let mut builder = RenderGraphBuilder::new("declaration-kind");
        let texture = builder.create_texture(TextureDesc::new(
            "scene-color",
            16,
            16,
            TextureFormat::Rgba8Unorm,
            TextureUsage::RENDER_ATTACHMENT,
        ));
        let buffer = builder.create_buffer(BufferDesc::new(
            "light-list",
            64,
            BufferUsage::STORAGE | BufferUsage::COPY_DST,
        ));
        let output = builder.import_external_resource("viewport-output");
        let pass = builder.add_pass("write", QueueLane::Graphics);
        builder.write_texture(pass, texture).unwrap();
        builder.write_buffer(pass, buffer).unwrap();
        builder.write_external(pass, output).unwrap();
        let graph = builder.compile().unwrap();
        let resources = RenderGraphExecutionResources::new();
        let texture_declaration = graph
            .resource_declaration(RenderGraphResource::TransientTexture(texture))
            .unwrap();
        let buffer_declaration = graph
            .resource_declaration(RenderGraphResource::TransientBuffer(buffer))
            .unwrap();

        assert_eq!(
            texture_declaration.kind,
            RenderGraphResourceKind::TransientTexture
        );
        assert_eq!(
            resources
                .require_buffer_for_declaration(texture_declaration)
                .unwrap_err(),
            "render graph execution resource `scene-color` is a texture declaration, not a buffer"
        );
        assert_eq!(
            buffer_declaration.kind,
            RenderGraphResourceKind::TransientBuffer
        );
        assert_eq!(
            resources
                .require_texture_view_for_declaration(buffer_declaration)
                .unwrap_err(),
            "render graph execution resource `light-list` is a buffer declaration, not a texture view"
        );
    }
}
