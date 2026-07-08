use crate::render_graph::CompiledRenderGraph;
use crate::rhi::{
    BufferDesc, BufferUsage, TextureDesc, TextureDimension, TextureFormat, TextureUsage,
};

use super::{
    render_graph_execution_resources::RenderGraphExecutionResources, TransientResourcePool,
};

pub(super) fn materialize_transient_resources(
    resources: &mut RenderGraphExecutionResources,
    device: &wgpu::Device,
    graph: &CompiledRenderGraph,
    mut pool: Option<&mut TransientResourcePool>,
) -> Result<(), String> {
    // Compiled lifetimes only include live passes, so culled scratch writers
    // never receive concrete WGPU backing.
    let lifetimes = graph.resource_lifetimes();
    super::transient_materialization::materialize_transient_texture_slots(
        resources,
        device,
        graph,
        pool.as_deref_mut(),
    )?;
    super::transient_materialization::materialize_transient_buffer_slots(
        resources,
        device,
        graph,
        pool.as_deref_mut(),
    )?;
    for lifetime in lifetimes {
        if lifetime.imported {
            continue;
        }
        let Some((parent, mip_level)) =
            super::transient_materialization::ssr_pyramid_mip_alias_for_lifetimes(
                lifetimes,
                &lifetime.name,
            )
        else {
            continue;
        };
        let view = resources.owned_texture_mip_view(parent, mip_level)?;
        resources.import_texture_view(lifetime.name.clone(), view);
    }
    Ok(())
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

pub(super) fn wgpu_texture_format(format: TextureFormat) -> wgpu::TextureFormat {
    match format {
        TextureFormat::R8Unorm => wgpu::TextureFormat::R8Unorm,
        TextureFormat::R16Float => wgpu::TextureFormat::R16Float,
        TextureFormat::R32Float => wgpu::TextureFormat::R32Float,
        TextureFormat::Rg16Float => wgpu::TextureFormat::Rg16Float,
        TextureFormat::Rg11b10Ufloat => wgpu::TextureFormat::Rg11b10Ufloat,
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

pub(super) fn wgpu_texture_usages(
    format: TextureFormat,
    usage: TextureUsage,
) -> wgpu::TextureUsages {
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
        TextureFormat::R32Float
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
mod tests;
