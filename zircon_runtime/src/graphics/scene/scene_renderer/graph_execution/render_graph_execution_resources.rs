use std::collections::BTreeMap;

use crate::core::framework::render::{
    PostProcessGraphResourceNames, RenderGraphExecutionResourceReport,
};
use crate::render_graph::{
    CompiledRenderGraph, RenderGraphResourceDeclaration, RenderGraphResourceDesc,
    RenderGraphResourceKind, RenderGraphResourceLifetime,
};
use crate::rhi::{
    BufferDesc, BufferUsage, TextureDesc, TextureDimension, TextureFormat, TextureUsage,
};

use super::TransientResourcePool;

#[derive(Clone, Copy, Debug)]
pub(in crate::graphics::scene::scene_renderer) struct RenderGraphImportedFinalTarget<'a> {
    pub view: &'a wgpu::TextureView,
}

#[derive(Default, Debug)]
pub struct RenderGraphExecutionResources {
    imported_texture_views: BTreeMap<String, wgpu::TextureView>,
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

    pub fn insert_buffer(
        &mut self,
        name: impl Into<String>,
        buffer: wgpu::Buffer,
    ) -> Option<wgpu::Buffer> {
        let name = name.into();
        self.buffer_backings.insert(name.clone(), name.clone());
        self.buffers.insert(name, buffer)
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer) fn materialize_transient_resources(
        &mut self,
        device: &wgpu::Device,
        graph: &CompiledRenderGraph,
    ) -> Result<(), String> {
        self.materialize_transient_resources_internal(device, graph, None)
    }

    pub(in crate::graphics::scene::scene_renderer) fn materialize_transient_resources_with_pool(
        &mut self,
        device: &wgpu::Device,
        graph: &CompiledRenderGraph,
        pool: &mut TransientResourcePool,
    ) -> Result<(), String> {
        self.materialize_transient_resources_internal(device, graph, Some(pool))
    }

    fn materialize_transient_resources_internal(
        &mut self,
        device: &wgpu::Device,
        graph: &CompiledRenderGraph,
        mut pool: Option<&mut TransientResourcePool>,
    ) -> Result<(), String> {
        // Compiled lifetimes only include live passes, so culled scratch writers
        // never receive concrete WGPU backing.
        let lifetimes = graph.resource_lifetimes();
        self.materialize_transient_texture_slots(device, graph, pool.as_deref_mut())?;
        self.materialize_transient_buffer_slots(device, graph, pool.as_deref_mut())?;
        for lifetime in lifetimes {
            if lifetime.imported {
                continue;
            }
            let Some((parent, mip_level)) =
                ssr_pyramid_mip_alias_for_lifetimes(lifetimes, &lifetime.name)
            else {
                continue;
            };
            let view = self.owned_texture_mip_view(parent, mip_level)?;
            self.import_texture_view(lifetime.name.clone(), view);
        }
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn release_transient_backings_into_pool(
        &mut self,
        pool: &mut TransientResourcePool,
    ) {
        self.imported_texture_views.clear();
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

    pub fn texture_view(&self, name: &str) -> Option<&wgpu::TextureView> {
        self.imported_texture_views.get(name)
    }

    pub fn buffer(&self, name: &str) -> Option<&wgpu::Buffer> {
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

    pub(in crate::graphics::scene::scene_renderer) fn owned_texture_mip_level_count(
        &self,
        name: &str,
    ) -> Option<u32> {
        self.owned_texture_descs
            .get(self.owned_texture_backing(name).unwrap_or(name))
            .map(|desc| desc.mip_levels)
    }

    pub fn require_texture_view(&self, name: &str) -> Result<&wgpu::TextureView, String> {
        self.texture_view(name)
            .ok_or_else(|| format!("render graph execution texture resource `{name}` is not bound"))
    }

    pub fn require_texture_view_for_declaration(
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

    pub fn require_buffer(&self, name: &str) -> Result<&wgpu::Buffer, String> {
        self.buffer(name)
            .ok_or_else(|| format!("render graph execution buffer resource `{name}` is not bound"))
    }

    pub fn require_buffer_for_declaration(
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

    fn is_owned_backed_texture_view(&self, name: &str) -> bool {
        self.owned_texture_backings.contains_key(name)
            || ssr_pyramid_mip_alias(name)
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

    fn insert_owned_texture(
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

    fn insert_owned_texture_backing(
        &mut self,
        backing_name: impl Into<String>,
        texture: wgpu::Texture,
        desc: TextureDesc,
    ) -> Option<wgpu::Texture> {
        let backing_name = backing_name.into();
        self.owned_texture_descs.insert(backing_name.clone(), desc);
        self.owned_textures.insert(backing_name, texture)
    }

    fn bind_owned_texture_view(
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

    fn insert_buffer_backing(
        &mut self,
        backing_name: impl Into<String>,
        buffer: wgpu::Buffer,
        desc: BufferDesc,
    ) -> Option<wgpu::Buffer> {
        let backing_name = backing_name.into();
        self.owned_buffer_descs.insert(backing_name.clone(), desc);
        self.buffers.insert(backing_name, buffer)
    }

    fn bind_buffer(
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

    fn materialize_transient_texture_slots(
        &mut self,
        device: &wgpu::Device,
        graph: &CompiledRenderGraph,
        mut pool: Option<&mut TransientResourcePool>,
    ) -> Result<(), String> {
        let lifetimes = graph.resource_lifetimes();
        let allocation_plan = graph.transient_allocation_plan();
        let lifetime_by_name = lifetimes
            .iter()
            .map(|lifetime| (lifetime.name.as_str(), lifetime))
            .collect::<BTreeMap<_, _>>();
        let mut slot_lifetimes = BTreeMap::<usize, Vec<&RenderGraphResourceLifetime>>::new();

        for allocation in allocation_plan
            .allocations
            .iter()
            .filter(|allocation| allocation.kind == RenderGraphResourceKind::TransientTexture)
        {
            let Some(lifetime) = lifetime_by_name.get(allocation.resource_name.as_str()) else {
                continue;
            };
            if !self.should_materialize_texture_lifetime(lifetimes, lifetime)? {
                continue;
            }
            slot_lifetimes
                .entry(allocation.slot)
                .or_default()
                .push(*lifetime);
        }

        for (slot, lifetimes) in slot_lifetimes {
            if let Some(desc) = compatible_texture_slot_desc(slot, &lifetimes)? {
                let backing_name = format!("rg-transient-texture-slot-{slot}");
                self.insert_owned_texture_backing(
                    backing_name.clone(),
                    acquire_wgpu_texture(device, &desc, pool.as_deref_mut()),
                    desc,
                );
                for lifetime in lifetimes {
                    self.bind_owned_texture_view(lifetime.name.clone(), &backing_name)?;
                }
            } else {
                for lifetime in lifetimes {
                    let RenderGraphResourceDesc::Texture(desc) = &lifetime.desc else {
                        return Err(format!(
                            "render graph resource `{}` has mismatched lifetime kind and descriptor",
                            lifetime.name
                        ));
                    };
                    self.insert_owned_texture(
                        lifetime.name.clone(),
                        acquire_wgpu_texture(device, desc, pool.as_deref_mut()),
                        desc.clone(),
                    );
                }
            }
        }
        Ok(())
    }

    fn materialize_transient_buffer_slots(
        &mut self,
        device: &wgpu::Device,
        graph: &CompiledRenderGraph,
        mut pool: Option<&mut TransientResourcePool>,
    ) -> Result<(), String> {
        let lifetimes = graph.resource_lifetimes();
        let allocation_plan = graph.transient_allocation_plan();
        let lifetime_by_name = lifetimes
            .iter()
            .map(|lifetime| (lifetime.name.as_str(), lifetime))
            .collect::<BTreeMap<_, _>>();
        let mut slot_lifetimes = BTreeMap::<usize, Vec<&RenderGraphResourceLifetime>>::new();

        for allocation in allocation_plan
            .allocations
            .iter()
            .filter(|allocation| allocation.kind == RenderGraphResourceKind::TransientBuffer)
        {
            let Some(lifetime) = lifetime_by_name.get(allocation.resource_name.as_str()) else {
                continue;
            };
            if lifetime.imported || self.has_buffer(&lifetime.name) {
                continue;
            }
            slot_lifetimes
                .entry(allocation.slot)
                .or_default()
                .push(*lifetime);
        }

        for (slot, lifetimes) in slot_lifetimes {
            let desc = buffer_slot_desc(slot, &lifetimes)?;
            let backing_name = format!("rg-transient-buffer-slot-{slot}");
            self.insert_buffer_backing(
                backing_name.clone(),
                acquire_wgpu_buffer(device, &desc, pool.as_deref_mut()),
                desc.clone(),
            );
            for lifetime in lifetimes {
                self.bind_buffer(lifetime.name.clone(), &backing_name);
            }
        }
        Ok(())
    }

    fn should_materialize_texture_lifetime(
        &self,
        lifetimes: &[RenderGraphResourceLifetime],
        lifetime: &RenderGraphResourceLifetime,
    ) -> Result<bool, String> {
        if lifetime.imported || self.has_texture_view(&lifetime.name) {
            return Ok(false);
        }
        let RenderGraphResourceDesc::Texture(desc) = &lifetime.desc else {
            return Err(format!(
                "render graph resource `{}` has mismatched lifetime kind and descriptor",
                lifetime.name
            ));
        };
        Ok(!desc.is_sparse_reserved()
            && ssr_pyramid_mip_alias_for_lifetimes(lifetimes, &lifetime.name).is_none())
    }
}

fn compatible_texture_slot_desc(
    slot: usize,
    lifetimes: &[&RenderGraphResourceLifetime],
) -> Result<Option<TextureDesc>, String> {
    let Some(first) = lifetimes.first() else {
        return Ok(None);
    };
    let RenderGraphResourceDesc::Texture(first_desc) = &first.desc else {
        return Err(format!(
            "render graph resource `{}` has mismatched lifetime kind and descriptor",
            first.name
        ));
    };
    let mut desc = first_desc.clone();
    desc.label = Some(format!("rg-transient-texture-slot-{slot}"));

    for lifetime in lifetimes.iter().skip(1) {
        let RenderGraphResourceDesc::Texture(next) = &lifetime.desc else {
            return Err(format!(
                "render graph resource `{}` has mismatched lifetime kind and descriptor",
                lifetime.name
            ));
        };
        if !texture_descs_can_share_wgpu_backing(&desc, next) {
            return Ok(None);
        }
        desc.usage |= next.usage;
    }

    Ok(Some(desc))
}

fn texture_descs_can_share_wgpu_backing(left: &TextureDesc, right: &TextureDesc) -> bool {
    left.width == right.width
        && left.height == right.height
        && left.depth == right.depth
        && left.mip_levels == right.mip_levels
        && left.sample_count == right.sample_count
        && left.format == right.format
        && left.dimension == right.dimension
        && left.residency == right.residency
}

fn buffer_slot_desc(
    slot: usize,
    lifetimes: &[&RenderGraphResourceLifetime],
) -> Result<BufferDesc, String> {
    let Some(first) = lifetimes.first() else {
        return Err(format!(
            "render graph transient buffer slot `{slot}` has no logical resources"
        ));
    };
    let RenderGraphResourceDesc::Buffer(first_desc) = &first.desc else {
        return Err(format!(
            "render graph resource `{}` has mismatched lifetime kind and descriptor",
            first.name
        ));
    };
    let mut desc = BufferDesc::new(
        format!("rg-transient-buffer-slot-{slot}"),
        first_desc.size_bytes,
        first_desc.usage,
    );

    for lifetime in lifetimes.iter().skip(1) {
        let RenderGraphResourceDesc::Buffer(next) = &lifetime.desc else {
            return Err(format!(
                "render graph resource `{}` has mismatched lifetime kind and descriptor",
                lifetime.name
            ));
        };
        desc.size_bytes = desc.size_bytes.max(next.size_bytes);
        desc.usage |= next.usage;
    }

    Ok(desc)
}

fn acquire_wgpu_texture(
    device: &wgpu::Device,
    desc: &TextureDesc,
    pool: Option<&mut TransientResourcePool>,
) -> wgpu::Texture {
    match pool {
        Some(pool) => pool.acquire_texture(device, desc),
        None => create_wgpu_texture(device, desc),
    }
}

fn acquire_wgpu_buffer(
    device: &wgpu::Device,
    desc: &BufferDesc,
    pool: Option<&mut TransientResourcePool>,
) -> wgpu::Buffer {
    match pool {
        Some(pool) => pool.acquire_buffer(device, desc),
        None => create_wgpu_buffer(device, desc),
    }
}

fn ssr_pyramid_mip_alias_for_lifetimes<'a>(
    lifetimes: &'a [crate::render_graph::RenderGraphResourceLifetime],
    name: &str,
) -> Option<(&'static str, u32)> {
    let (parent, mip_level) = ssr_pyramid_mip_alias(name)?;
    lifetimes
        .iter()
        .find(|lifetime| lifetime.name == parent)
        .and_then(|lifetime| match &lifetime.desc {
            RenderGraphResourceDesc::Texture(desc) if desc.mip_levels > mip_level => {
                Some((parent, mip_level))
            }
            _ => None,
        })
}

fn ssr_pyramid_mip_alias(name: &str) -> Option<(&'static str, u32)> {
    match name {
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_DEPTH_PYRAMID_COARSE => Some((
            PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_DEPTH_PYRAMID,
            1,
        )),
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE => Some((
            PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID,
            1,
        )),
        _ => None,
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

pub(super) fn create_wgpu_texture(device: &wgpu::Device, desc: &TextureDesc) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: desc.label.as_deref(),
        size: wgpu::Extent3d {
            width: desc.width,
            height: desc.height,
            depth_or_array_layers: desc.depth,
        },
        mip_level_count: desc.mip_levels,
        sample_count: desc.sample_count,
        dimension: wgpu_texture_dimension(desc.dimension),
        format: wgpu_texture_format(desc.format),
        usage: wgpu_texture_usages(desc.format, desc.usage),
        view_formats: &[],
    })
}

pub(super) fn create_wgpu_buffer(device: &wgpu::Device, desc: &BufferDesc) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: desc.label.as_deref(),
        size: desc.size_bytes,
        usage: wgpu_buffer_usages(desc.usage),
        mapped_at_creation: false,
    })
}

fn wgpu_texture_dimension(dimension: TextureDimension) -> wgpu::TextureDimension {
    match dimension {
        TextureDimension::D1 => wgpu::TextureDimension::D1,
        TextureDimension::D2 | TextureDimension::D2Array | TextureDimension::Cube => {
            wgpu::TextureDimension::D2
        }
        TextureDimension::D3 => wgpu::TextureDimension::D3,
    }
}

fn wgpu_texture_format(format: TextureFormat) -> wgpu::TextureFormat {
    match format {
        TextureFormat::R8Unorm => wgpu::TextureFormat::R8Unorm,
        TextureFormat::R16Float => wgpu::TextureFormat::R16Float,
        TextureFormat::R32Float => wgpu::TextureFormat::R32Float,
        TextureFormat::Rg16Float => wgpu::TextureFormat::Rg16Float,
        TextureFormat::Rgba8Unorm => wgpu::TextureFormat::Rgba8Unorm,
        TextureFormat::Rgba8UnormSrgb => wgpu::TextureFormat::Rgba8UnormSrgb,
        TextureFormat::Bgra8Unorm => wgpu::TextureFormat::Bgra8Unorm,
        TextureFormat::Bgra8UnormSrgb => wgpu::TextureFormat::Bgra8UnormSrgb,
        TextureFormat::Rgba16Float => wgpu::TextureFormat::Rgba16Float,
        TextureFormat::Rgba32Float => wgpu::TextureFormat::Rgba32Float,
        TextureFormat::Depth24Plus => wgpu::TextureFormat::Depth24Plus,
        TextureFormat::Depth24PlusStencil8 => wgpu::TextureFormat::Depth24PlusStencil8,
        TextureFormat::Depth32Float => wgpu::TextureFormat::Depth32Float,
    }
}

fn wgpu_texture_usages(format: TextureFormat, usage: TextureUsage) -> wgpu::TextureUsages {
    let mut usages = wgpu::TextureUsages::empty();
    if usage.contains(TextureUsage::RENDER_ATTACHMENT) || usage.contains(TextureUsage::PRESENT) {
        usages |= wgpu::TextureUsages::RENDER_ATTACHMENT;
    }
    if usage.contains(TextureUsage::SAMPLED) {
        usages |= wgpu::TextureUsages::TEXTURE_BINDING;
    }
    if usage.contains(TextureUsage::STORAGE) && supports_storage_binding_usage(format) {
        usages |= wgpu::TextureUsages::STORAGE_BINDING;
    }
    if usage.contains(TextureUsage::COPY_SRC) {
        usages |= wgpu::TextureUsages::COPY_SRC;
    }
    if usage.contains(TextureUsage::COPY_DST) {
        usages |= wgpu::TextureUsages::COPY_DST;
    }
    usages
}

fn supports_storage_binding_usage(format: TextureFormat) -> bool {
    matches!(
        format,
        TextureFormat::R8Unorm
            | TextureFormat::R16Float
            | TextureFormat::R32Float
            | TextureFormat::Rg16Float
            | TextureFormat::Rgba8Unorm
            | TextureFormat::Rgba16Float
            | TextureFormat::Rgba32Float
    )
}

fn wgpu_buffer_usages(usage: BufferUsage) -> wgpu::BufferUsages {
    let mut usages = wgpu::BufferUsages::empty();
    if usage.contains(BufferUsage::VERTEX) {
        usages |= wgpu::BufferUsages::VERTEX;
    }
    if usage.contains(BufferUsage::INDEX) {
        usages |= wgpu::BufferUsages::INDEX;
    }
    if usage.contains(BufferUsage::UNIFORM) {
        usages |= wgpu::BufferUsages::UNIFORM;
    }
    if usage.contains(BufferUsage::STORAGE) {
        usages |= wgpu::BufferUsages::STORAGE;
    }
    if usage.contains(BufferUsage::STAGING_READ) {
        usages |= wgpu::BufferUsages::MAP_READ;
    }
    if usage.contains(BufferUsage::STAGING_WRITE) {
        usages |= wgpu::BufferUsages::MAP_WRITE;
    }
    if usage.contains(BufferUsage::INDIRECT) {
        usages |= wgpu::BufferUsages::INDIRECT;
    }
    if usage.contains(BufferUsage::COPY_SRC) {
        usages |= wgpu::BufferUsages::COPY_SRC;
    }
    if usage.contains(BufferUsage::COPY_DST) {
        usages |= wgpu::BufferUsages::COPY_DST;
    }
    usages
}

#[cfg(test)]
mod tests {
    use super::RenderGraphExecutionResources;
    use crate::core::framework::render::PostProcessGraphResourceNames;
    use crate::graphics::backend::RenderBackend;
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

    #[test]
    fn materialization_creates_dense_transients_and_skips_sparse_reservations() {
        let backend = RenderBackend::new_offscreen().unwrap();
        let mut builder = RenderGraphBuilder::new("materialization");
        let shadow = builder.create_texture(TextureDesc::new(
            "shadow-map",
            64,
            64,
            TextureFormat::Depth32Float,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
        ));
        let sparse = builder.create_texture(
            TextureDesc::new(
                "sparse-pages",
                128,
                128,
                TextureFormat::Rgba8Unorm,
                TextureUsage::SAMPLED | TextureUsage::STORAGE,
            )
            .with_sparse_residency(),
        );
        let scratch = builder.create_buffer(BufferDesc::new(
            "scratch",
            16,
            BufferUsage::STORAGE | BufferUsage::COPY_DST,
        ));
        let pass = builder.add_pass("materialize", QueueLane::Graphics);
        builder
            .set_pass_flags(
                pass,
                PassFlags {
                    allow_culling: true,
                    has_side_effects: true,
                },
            )
            .unwrap();
        builder.write_texture(pass, shadow).unwrap();
        builder.write_storage_texture(pass, sparse).unwrap();
        builder.write_buffer(pass, scratch).unwrap();
        let graph = builder.compile().unwrap();
        let mut resources = RenderGraphExecutionResources::new();

        resources
            .materialize_transient_resources(&backend.device, &graph)
            .unwrap();

        assert!(resources.has_texture_view("shadow-map"));
        assert!(
            !resources.has_texture_view("sparse-pages"),
            "sparse reservations must not be silently backed by a dense WGPU texture"
        );
        assert!(resources.has_buffer("scratch"));
        assert!(resources.has_bound_resource("shadow-map"));
        assert!(resources.has_bound_resource("scratch"));
        assert!(!resources.has_bound_resource("sparse-pages"));
        assert_eq!(
            resources.resource_report(),
            crate::core::framework::render::RenderGraphExecutionResourceReport::new(1, 0, 1, 1)
        );
    }

    #[test]
    fn materialization_aliases_compatible_transient_texture_slots() {
        let backend = RenderBackend::new_offscreen().unwrap();
        let mut builder = RenderGraphBuilder::new("compatible-texture-aliasing");
        let first = builder.create_texture(TextureDesc::new(
            "first-color",
            32,
            32,
            TextureFormat::Rgba8Unorm,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
        ));
        let second = builder.create_texture(TextureDesc::new(
            "second-color",
            32,
            32,
            TextureFormat::Rgba8Unorm,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
        ));
        let output = builder.import_external_resource("viewport-output");
        let first_write = builder.add_pass("first-write", QueueLane::Graphics);
        let first_read = builder.add_pass("first-read", QueueLane::Graphics);
        let second_write = builder.add_pass("second-write", QueueLane::Graphics);
        let second_read = builder.add_pass("second-read", QueueLane::Graphics);
        builder.write_texture(first_write, first).unwrap();
        builder.read_texture(first_read, first).unwrap();
        builder.write_texture(second_write, second).unwrap();
        builder.read_texture(second_read, second).unwrap();
        builder.write_external(second_read, output).unwrap();
        builder.add_dependency(first_read, second_write).unwrap();
        let graph = builder.compile().unwrap();
        let mut resources = RenderGraphExecutionResources::new();

        resources
            .materialize_transient_resources(&backend.device, &graph)
            .unwrap();

        assert_eq!(graph.transient_allocation_plan().texture_slot_count, 1);
        assert!(resources.has_texture_view("first-color"));
        assert!(resources.has_texture_view("second-color"));
        assert!(resources.owned_texture("first-color").is_some());
        assert!(resources.owned_texture("second-color").is_some());
        let report = resources.resource_report();
        assert_eq!(
            report.owned_texture_count, 1,
            "compatible non-overlapping logical textures should share one WGPU backing texture"
        );
        assert_eq!(report.external_texture_view_count, 0);
        assert_eq!(report.texture_view_count, 2);
    }

    #[test]
    fn materialization_keeps_incompatible_texture_slot_resources_separate() {
        let backend = RenderBackend::new_offscreen().unwrap();
        let mut builder = RenderGraphBuilder::new("incompatible-texture-aliasing");
        let large = builder.create_texture(TextureDesc::new(
            "large-color",
            64,
            64,
            TextureFormat::Rgba8Unorm,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
        ));
        let small = builder.create_texture(TextureDesc::new(
            "small-color",
            16,
            16,
            TextureFormat::Rgba8Unorm,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
        ));
        let output = builder.import_external_resource("viewport-output");
        let large_write = builder.add_pass("large-write", QueueLane::Graphics);
        let large_read = builder.add_pass("large-read", QueueLane::Graphics);
        let small_write = builder.add_pass("small-write", QueueLane::Graphics);
        let small_read = builder.add_pass("small-read", QueueLane::Graphics);
        builder.write_texture(large_write, large).unwrap();
        builder.read_texture(large_read, large).unwrap();
        builder.write_texture(small_write, small).unwrap();
        builder.read_texture(small_read, small).unwrap();
        builder.write_external(small_read, output).unwrap();
        builder.add_dependency(large_read, small_write).unwrap();
        let graph = builder.compile().unwrap();
        let mut resources = RenderGraphExecutionResources::new();

        resources
            .materialize_transient_resources(&backend.device, &graph)
            .unwrap();

        assert_eq!(
            graph.transient_allocation_plan().texture_slot_count,
            1,
            "the neutral byte plan may reserve one slot even when concrete WGPU textures need separate descriptors"
        );
        assert!(resources.has_texture_view("large-color"));
        assert!(resources.has_texture_view("small-color"));
        let report = resources.resource_report();
        assert_eq!(
            report.owned_texture_count, 2,
            "WGPU backing must not alias logical textures with different extents"
        );
        assert_eq!(report.external_texture_view_count, 0);
        assert_eq!(report.texture_view_count, 2);
    }

    #[test]
    fn materialization_aliases_transient_buffer_slots() {
        let backend = RenderBackend::new_offscreen().unwrap();
        let mut builder = RenderGraphBuilder::new("compatible-buffer-aliasing");
        let first = builder.create_buffer(BufferDesc::new(
            "first-indirect",
            64,
            BufferUsage::STORAGE | BufferUsage::COPY_DST,
        ));
        let second = builder.create_buffer(BufferDesc::new(
            "second-indirect",
            128,
            BufferUsage::STORAGE | BufferUsage::INDIRECT,
        ));
        let output = builder.import_external_resource("viewport-output");
        let first_write = builder.add_pass("first-buffer-write", QueueLane::Graphics);
        let first_read = builder.add_pass("first-buffer-read", QueueLane::Graphics);
        let second_write = builder.add_pass("second-buffer-write", QueueLane::Graphics);
        let second_read = builder.add_pass("second-buffer-read", QueueLane::Graphics);
        builder.write_buffer(first_write, first).unwrap();
        builder.read_buffer(first_read, first).unwrap();
        builder.write_buffer(second_write, second).unwrap();
        builder.read_buffer(second_read, second).unwrap();
        builder.write_external(second_read, output).unwrap();
        builder.add_dependency(first_read, second_write).unwrap();
        let graph = builder.compile().unwrap();
        let mut resources = RenderGraphExecutionResources::new();

        resources
            .materialize_transient_resources(&backend.device, &graph)
            .unwrap();

        assert_eq!(graph.transient_allocation_plan().buffer_slot_count, 1);
        assert!(resources.has_buffer("first-indirect"));
        assert!(resources.has_buffer("second-indirect"));
        let report = resources.resource_report();
        assert_eq!(
            report.buffer_count, 1,
            "compatible non-overlapping logical buffers should share one WGPU backing buffer"
        );
        assert_eq!(report.texture_view_count, 0);
        assert_eq!(report.total_bound_resource_count, 1);
    }

    #[test]
    fn materialization_exposes_owned_texture_mip_views() {
        let backend = RenderBackend::new_offscreen().unwrap();
        let mut builder = RenderGraphBuilder::new("mipped-materialization");
        let pyramid = builder.create_texture(
            TextureDesc::new(
                "mipped-pyramid",
                64,
                32,
                TextureFormat::Rgba16Float,
                TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
            )
            .with_mip_levels(3),
        );
        let pass = builder.add_pass("write-mip-zero", QueueLane::Graphics);
        builder.write_texture(pass, pyramid).unwrap();
        let graph = builder.compile().unwrap();
        let mut resources = RenderGraphExecutionResources::new();

        resources
            .materialize_transient_resources(&backend.device, &graph)
            .unwrap();

        assert!(resources.has_texture_view("mipped-pyramid"));
        assert!(resources
            .owned_texture_mip_view("mipped-pyramid", 1)
            .is_ok());
        assert_eq!(
            resources
                .owned_texture_mip_view("mipped-pyramid", 3)
                .unwrap_err(),
            "render graph execution texture resource `mipped-pyramid` mip level 3 is outside mip_levels 3"
        );
    }

    #[test]
    fn materialization_aliases_ssr_coarse_pyramids_to_parent_mip_views() {
        let backend = RenderBackend::new_offscreen().unwrap();
        let mut builder = RenderGraphBuilder::new("ssr-mip-aliases");
        let depth_pyramid = builder.create_texture(
            TextureDesc::new(
                PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_DEPTH_PYRAMID,
                64,
                32,
                TextureFormat::Rgba16Float,
                TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
            )
            .with_mip_levels(3),
        );
        let depth_pyramid_coarse = builder.create_texture(TextureDesc::new(
            PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_DEPTH_PYRAMID_COARSE,
            32,
            16,
            TextureFormat::Rgba16Float,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
        ));
        let reflection_pyramid = builder.create_texture(
            TextureDesc::new(
                PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID,
                64,
                32,
                TextureFormat::Rgba16Float,
                TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
            )
            .with_mip_levels(3),
        );
        let reflection_pyramid_coarse = builder.create_texture(TextureDesc::new(
            PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE,
            32,
            16,
            TextureFormat::Rgba16Float,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
        ));
        let output = builder.import_external_resource("viewport-output");
        let depth_pass = builder.add_pass("depth-pyramid", QueueLane::Graphics);
        builder.write_texture(depth_pass, depth_pyramid).unwrap();
        let depth_coarse_pass = builder.add_pass("depth-pyramid-coarse", QueueLane::Graphics);
        builder
            .read_texture(depth_coarse_pass, depth_pyramid)
            .unwrap();
        builder
            .write_texture(depth_coarse_pass, depth_pyramid_coarse)
            .unwrap();
        let reflection_pass = builder.add_pass("reflection-pyramid", QueueLane::Graphics);
        builder
            .write_texture(reflection_pass, reflection_pyramid)
            .unwrap();
        let reflection_coarse_pass =
            builder.add_pass("reflection-pyramid-coarse", QueueLane::Graphics);
        builder
            .read_texture(reflection_coarse_pass, reflection_pyramid)
            .unwrap();
        builder
            .write_texture(reflection_coarse_pass, reflection_pyramid_coarse)
            .unwrap();
        let output_pass = builder.add_pass("output", QueueLane::Graphics);
        builder
            .read_texture(output_pass, depth_pyramid_coarse)
            .unwrap();
        builder
            .read_texture(output_pass, reflection_pyramid_coarse)
            .unwrap();
        builder.write_external(output_pass, output).unwrap();
        let graph = builder.compile().unwrap();
        let mut resources = RenderGraphExecutionResources::new();

        resources
            .materialize_transient_resources(&backend.device, &graph)
            .unwrap();

        assert!(resources.has_texture_view(
            PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_DEPTH_PYRAMID
        ));
        assert!(resources.has_texture_view(
            PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_DEPTH_PYRAMID_COARSE
        ));
        assert!(resources.has_texture_view(
            PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID
        ));
        assert!(resources.has_texture_view(
            PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE
        ));
        assert!(resources
            .owned_texture(PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_DEPTH_PYRAMID)
            .is_some());
        assert!(resources
            .owned_texture(
                PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_DEPTH_PYRAMID_COARSE
            )
            .is_none());
        assert!(resources
            .owned_texture(
                PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID
            )
            .is_some());
        assert!(resources
            .owned_texture(
                PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE
            )
            .is_none());
        let report = resources.resource_report();
        assert_eq!(report.external_texture_view_count, 0);
        assert_eq!(report.owned_texture_count, 2);
        assert_eq!(report.texture_view_count, 4);
    }

    #[test]
    fn materialization_allocates_ssr_coarse_resource_when_parent_has_no_coarse_mip() {
        let backend = RenderBackend::new_offscreen().unwrap();
        let mut builder = RenderGraphBuilder::new("ssr-small-pyramid");
        let depth_pyramid = builder.create_texture(TextureDesc::new(
            PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_DEPTH_PYRAMID,
            1,
            1,
            TextureFormat::Rgba16Float,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
        ));
        let depth_pyramid_coarse = builder.create_texture(TextureDesc::new(
            PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_DEPTH_PYRAMID_COARSE,
            1,
            1,
            TextureFormat::Rgba16Float,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
        ));
        let output = builder.import_external_resource("viewport-output");
        let depth_pass = builder.add_pass("depth-pyramid", QueueLane::Graphics);
        builder.write_texture(depth_pass, depth_pyramid).unwrap();
        let depth_coarse_pass = builder.add_pass("depth-pyramid-coarse", QueueLane::Graphics);
        builder
            .read_texture(depth_coarse_pass, depth_pyramid)
            .unwrap();
        builder
            .write_texture(depth_coarse_pass, depth_pyramid_coarse)
            .unwrap();
        let output_pass = builder.add_pass("output", QueueLane::Graphics);
        builder
            .read_texture(output_pass, depth_pyramid_coarse)
            .unwrap();
        builder.write_external(output_pass, output).unwrap();
        let graph = builder.compile().unwrap();
        let mut resources = RenderGraphExecutionResources::new();

        resources
            .materialize_transient_resources(&backend.device, &graph)
            .unwrap();

        assert!(resources
            .owned_texture(PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_DEPTH_PYRAMID)
            .is_some());
        assert!(resources
            .owned_texture(
                PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_DEPTH_PYRAMID_COARSE
            )
            .is_some());
    }
}
