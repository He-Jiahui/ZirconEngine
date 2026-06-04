use std::collections::BTreeMap;

use crate::render_graph::{CompiledRenderGraph, RenderGraphResourceDesc, RenderGraphResourceKind};
use crate::rhi::{
    BufferDesc, BufferUsage, TextureDesc, TextureDimension, TextureFormat, TextureUsage,
};

#[derive(Default, Debug)]
pub struct RenderGraphExecutionResources {
    imported_texture_views: BTreeMap<String, wgpu::TextureView>,
    owned_textures: BTreeMap<String, wgpu::Texture>,
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
        for lifetime in graph.resource_lifetimes() {
            if lifetime.imported {
                continue;
            }
            match (&lifetime.kind, &lifetime.desc) {
                (
                    RenderGraphResourceKind::TransientTexture,
                    RenderGraphResourceDesc::Texture(desc),
                ) => {
                    if self.has_texture_view(&lifetime.name) || desc.is_sparse_reserved() {
                        continue;
                    }
                    self.insert_owned_texture(
                        lifetime.name.clone(),
                        create_wgpu_texture(device, desc),
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
        Ok(())
    }

    pub fn texture_view(&self, name: &str) -> Option<&wgpu::TextureView> {
        self.imported_texture_views.get(name)
    }

    pub fn buffer(&self, name: &str) -> Option<&wgpu::Buffer> {
        self.buffers.get(name)
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
    ) -> Option<wgpu::Texture> {
        let name = name.into();
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.import_texture_view(name.clone(), view);
        self.owned_textures.insert(name, texture)
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
    }
}
