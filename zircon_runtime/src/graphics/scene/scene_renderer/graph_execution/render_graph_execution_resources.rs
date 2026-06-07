use std::collections::BTreeMap;

use crate::core::framework::render::{
    PostProcessGraphResourceNames, RenderGraphExecutionResourceReport,
};
use crate::render_graph::{CompiledRenderGraph, RenderGraphResourceDesc, RenderGraphResourceKind};
use crate::rhi::{
    BufferDesc, BufferUsage, TextureDesc, TextureDimension, TextureFormat, TextureUsage,
};

#[derive(Clone, Copy, Debug)]
pub(in crate::graphics::scene::scene_renderer) struct RenderGraphImportedFinalTarget<'a> {
    pub view: &'a wgpu::TextureView,
}

#[derive(Default, Debug)]
pub struct RenderGraphExecutionResources {
    imported_texture_views: BTreeMap<String, wgpu::TextureView>,
    owned_textures: BTreeMap<String, wgpu::Texture>,
    owned_texture_descs: BTreeMap<String, TextureDesc>,
    buffers: BTreeMap<String, wgpu::Buffer>,
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
        self.buffers.insert(name.into(), buffer)
    }

    pub(in crate::graphics::scene::scene_renderer) fn materialize_transient_resources(
        &mut self,
        device: &wgpu::Device,
        graph: &CompiledRenderGraph,
    ) -> Result<(), String> {
        // Compiled lifetimes only include live passes, so culled scratch writers
        // never receive concrete WGPU backing.
        let lifetimes = graph.resource_lifetimes();
        for lifetime in graph.resource_lifetimes() {
            if lifetime.imported {
                continue;
            }
            match (&lifetime.kind, &lifetime.desc) {
                (
                    RenderGraphResourceKind::TransientTexture,
                    RenderGraphResourceDesc::Texture(desc),
                ) => {
                    if self.has_texture_view(&lifetime.name)
                        || desc.is_sparse_reserved()
                        || ssr_pyramid_mip_alias_for_lifetimes(lifetimes, &lifetime.name).is_some()
                    {
                        continue;
                    }
                    self.insert_owned_texture(
                        lifetime.name.clone(),
                        create_wgpu_texture(device, desc),
                        desc.clone(),
                    );
                }
                (
                    RenderGraphResourceKind::TransientBuffer,
                    RenderGraphResourceDesc::Buffer(desc),
                ) => {
                    if self.has_buffer(&lifetime.name) {
                        continue;
                    }
                    self.insert_buffer(lifetime.name.clone(), create_wgpu_buffer(device, desc));
                }
                (RenderGraphResourceKind::External, RenderGraphResourceDesc::External) => {}
                _ => {
                    return Err(format!(
                        "render graph resource `{}` has mismatched lifetime kind and descriptor",
                        lifetime.name
                    ));
                }
            }
        }
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

    pub fn texture_view(&self, name: &str) -> Option<&wgpu::TextureView> {
        self.imported_texture_views.get(name)
    }

    pub fn buffer(&self, name: &str) -> Option<&wgpu::Buffer> {
        self.buffers.get(name)
    }

    pub(in crate::graphics::scene::scene_renderer) fn owned_texture(
        &self,
        name: &str,
    ) -> Option<&wgpu::Texture> {
        self.owned_textures.get(name)
    }

    pub(in crate::graphics::scene::scene_renderer) fn owned_texture_mip_view(
        &self,
        name: &str,
        mip_level: u32,
    ) -> Result<wgpu::TextureView, String> {
        let texture = self.owned_textures.get(name).ok_or_else(|| {
            format!(
                "render graph execution texture resource `{name}` is not an owned transient texture"
            )
        })?;
        let desc = self.owned_texture_descs.get(name).ok_or_else(|| {
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
        let texture = self.owned_textures.get(name).ok_or_else(|| {
            format!(
                "render graph execution texture resource `{name}` is not an owned transient texture"
            )
        })?;
        let desc = self.owned_texture_descs.get(name).ok_or_else(|| {
            format!("render graph execution texture resource `{name}` is missing its descriptor")
        })?;
        Ok(texture.create_view(&texture_full_mip_view_descriptor(desc.mip_levels)))
    }

    pub(in crate::graphics::scene::scene_renderer) fn owned_texture_mip_level_count(
        &self,
        name: &str,
    ) -> Option<u32> {
        self.owned_texture_descs
            .get(name)
            .map(|desc| desc.mip_levels)
    }

    pub fn require_texture_view(&self, name: &str) -> Result<&wgpu::TextureView, String> {
        self.texture_view(name)
            .ok_or_else(|| format!("render graph execution texture resource `{name}` is not bound"))
    }

    pub fn require_buffer(&self, name: &str) -> Result<&wgpu::Buffer, String> {
        self.buffer(name)
            .ok_or_else(|| format!("render graph execution buffer resource `{name}` is not bound"))
    }

    pub fn has_texture_view(&self, name: &str) -> bool {
        self.imported_texture_views.contains_key(name)
    }

    pub fn has_buffer(&self, name: &str) -> bool {
        self.buffers.contains_key(name)
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
        self.owned_textures.contains_key(name)
            || ssr_pyramid_mip_alias(name)
                .is_some_and(|(parent, _)| self.owned_textures.contains_key(parent))
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
        self.owned_texture_descs.insert(name.clone(), desc);
        self.owned_textures.insert(name, texture)
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

fn create_wgpu_texture(device: &wgpu::Device, desc: &TextureDesc) -> wgpu::Texture {
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

fn create_wgpu_buffer(device: &wgpu::Device, desc: &BufferDesc) -> wgpu::Buffer {
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
    use crate::render_graph::{PassFlags, QueueLane, RenderGraphBuilder};
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
    fn materialization_creates_dense_transients_and_skips_sparse_reservations() {
        let backend = RenderBackend::new_offscreen().unwrap();
        let mut builder = RenderGraphBuilder::new("materialization");
        let shadow = builder.create_transient_texture(TextureDesc::new(
            "shadow-map",
            64,
            64,
            TextureFormat::Depth32Float,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
        ));
        let sparse = builder.create_transient_texture(
            TextureDesc::new(
                "sparse-pages",
                128,
                128,
                TextureFormat::Rgba8Unorm,
                TextureUsage::SAMPLED | TextureUsage::STORAGE,
            )
            .with_sparse_residency(),
        );
        let scratch = builder.create_transient_buffer(BufferDesc::new(
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
    fn materialization_exposes_owned_texture_mip_views() {
        let backend = RenderBackend::new_offscreen().unwrap();
        let mut builder = RenderGraphBuilder::new("mipped-materialization");
        let pyramid = builder.create_transient_texture(
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
        let depth_pyramid = builder.create_transient_texture(
            TextureDesc::new(
                PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_DEPTH_PYRAMID,
                64,
                32,
                TextureFormat::Rgba16Float,
                TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
            )
            .with_mip_levels(3),
        );
        let depth_pyramid_coarse = builder.create_transient_texture(TextureDesc::new(
            PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_DEPTH_PYRAMID_COARSE,
            32,
            16,
            TextureFormat::Rgba16Float,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
        ));
        let reflection_pyramid = builder.create_transient_texture(
            TextureDesc::new(
                PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID,
                64,
                32,
                TextureFormat::Rgba16Float,
                TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
            )
            .with_mip_levels(3),
        );
        let reflection_pyramid_coarse = builder.create_transient_texture(TextureDesc::new(
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
        let depth_pyramid = builder.create_transient_texture(TextureDesc::new(
            PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_DEPTH_PYRAMID,
            1,
            1,
            TextureFormat::Rgba16Float,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
        ));
        let depth_pyramid_coarse = builder.create_transient_texture(TextureDesc::new(
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
