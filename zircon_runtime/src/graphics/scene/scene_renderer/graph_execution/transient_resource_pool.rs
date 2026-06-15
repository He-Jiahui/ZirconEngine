use std::collections::BTreeMap;

use crate::core::framework::render::RenderGraphTransientPoolReport;
use crate::rhi::{BufferDesc, TextureDesc, TextureDimension, TextureFormat, TextureResidency};

use super::render_graph_execution_resources::{create_wgpu_buffer, create_wgpu_texture};

pub(in crate::graphics::scene::scene_renderer) const TRANSIENT_RESOURCE_POOL_KEEP_FRAMES: u64 = 8;

#[derive(Default)]
pub(in crate::graphics::scene::scene_renderer) struct TransientResourcePool {
    frame_index: u64,
    frame_report: RenderGraphTransientPoolReport,
    last_frame_report: RenderGraphTransientPoolReport,
    textures: BTreeMap<TransientTextureKey, Vec<PooledTexture>>,
    buffers: BTreeMap<TransientBufferKey, Vec<PooledBuffer>>,
}

impl TransientResourcePool {
    pub fn begin_frame(&mut self) {
        self.frame_report = RenderGraphTransientPoolReport {
            frame_index: self.frame_index,
            texture_pool_entry_count: self.texture_entry_count(),
            buffer_pool_entry_count: self.buffer_entry_count(),
            ..Default::default()
        };
    }

    pub fn acquire_texture(&mut self, device: &wgpu::Device, desc: &TextureDesc) -> wgpu::Texture {
        let key = TransientTextureKey::from(desc);
        if let Some(texture) = self
            .textures
            .get_mut(&key)
            .and_then(|entries| entries.pop())
            .map(|entry| entry.texture)
        {
            self.frame_report.texture_reused_count += 1;
            return texture;
        }

        self.frame_report.texture_created_count += 1;
        create_wgpu_texture(device, desc)
    }

    pub fn release_texture(&mut self, desc: TextureDesc, texture: wgpu::Texture) {
        self.textures
            .entry(TransientTextureKey::from(&desc))
            .or_default()
            .push(PooledTexture {
                texture,
                last_used_frame: self.frame_index,
            });
    }

    pub fn acquire_buffer(&mut self, device: &wgpu::Device, desc: &BufferDesc) -> wgpu::Buffer {
        let key = TransientBufferKey::from(desc);
        if let Some(buffer) = self
            .buffers
            .get_mut(&key)
            .and_then(|entries| entries.pop())
            .map(|entry| entry.buffer)
        {
            self.frame_report.buffer_reused_count += 1;
            return buffer;
        }

        self.frame_report.buffer_created_count += 1;
        create_wgpu_buffer(device, desc)
    }

    pub fn release_buffer(&mut self, desc: BufferDesc, buffer: wgpu::Buffer) {
        self.buffers
            .entry(TransientBufferKey::from(&desc))
            .or_default()
            .push(PooledBuffer {
                buffer,
                last_used_frame: self.frame_index,
            });
    }

    pub fn end_frame(&mut self) {
        self.frame_index = self.frame_index.saturating_add(1);
        self.frame_report.evicted_texture_count = evict_stale_textures(
            &mut self.textures,
            self.frame_index,
            TRANSIENT_RESOURCE_POOL_KEEP_FRAMES,
        );
        self.frame_report.evicted_buffer_count = evict_stale_buffers(
            &mut self.buffers,
            self.frame_index,
            TRANSIENT_RESOURCE_POOL_KEEP_FRAMES,
        );
        self.frame_report.texture_pool_entry_count = self.texture_entry_count();
        self.frame_report.buffer_pool_entry_count = self.buffer_entry_count();
        self.last_frame_report = self.frame_report;
    }

    pub fn last_frame_report(&self) -> RenderGraphTransientPoolReport {
        self.last_frame_report
    }

    fn texture_entry_count(&self) -> usize {
        self.textures.values().map(Vec::len).sum()
    }

    fn buffer_entry_count(&self) -> usize {
        self.buffers.values().map(Vec::len).sum()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct TransientTextureKey {
    width: u32,
    height: u32,
    depth: u32,
    mip_levels: u32,
    sample_count: u32,
    format: u8,
    usage_bits: u32,
    dimension: u8,
    residency: u8,
}

impl From<&TextureDesc> for TransientTextureKey {
    fn from(desc: &TextureDesc) -> Self {
        Self {
            width: desc.width,
            height: desc.height,
            depth: desc.depth,
            mip_levels: desc.mip_levels,
            sample_count: desc.sample_count,
            format: texture_format_tag(desc.format),
            usage_bits: desc.usage.bits(),
            dimension: texture_dimension_tag(desc.dimension),
            residency: texture_residency_tag(desc.residency),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct TransientBufferKey {
    size_bytes: u64,
    usage_bits: u32,
}

impl From<&BufferDesc> for TransientBufferKey {
    fn from(desc: &BufferDesc) -> Self {
        Self {
            size_bytes: desc.size_bytes,
            usage_bits: desc.usage.bits(),
        }
    }
}

struct PooledTexture {
    texture: wgpu::Texture,
    last_used_frame: u64,
}

struct PooledBuffer {
    buffer: wgpu::Buffer,
    last_used_frame: u64,
}

fn evict_stale_textures(
    textures: &mut BTreeMap<TransientTextureKey, Vec<PooledTexture>>,
    frame_index: u64,
    keep_frames: u64,
) -> usize {
    let mut evicted = 0;
    textures.retain(|_, entries| {
        let before = entries.len();
        entries.retain(|entry| frame_index.saturating_sub(entry.last_used_frame) <= keep_frames);
        evicted += before.saturating_sub(entries.len());
        !entries.is_empty()
    });
    evicted
}

fn evict_stale_buffers(
    buffers: &mut BTreeMap<TransientBufferKey, Vec<PooledBuffer>>,
    frame_index: u64,
    keep_frames: u64,
) -> usize {
    let mut evicted = 0;
    buffers.retain(|_, entries| {
        let before = entries.len();
        entries.retain(|entry| frame_index.saturating_sub(entry.last_used_frame) <= keep_frames);
        evicted += before.saturating_sub(entries.len());
        !entries.is_empty()
    });
    evicted
}

fn texture_format_tag(format: TextureFormat) -> u8 {
    match format {
        TextureFormat::R8Unorm => 0,
        TextureFormat::R16Float => 1,
        TextureFormat::R32Float => 2,
        TextureFormat::Rg16Float => 3,
        TextureFormat::Rg11b10Ufloat => 4,
        TextureFormat::Rgba8Unorm => 5,
        TextureFormat::Rgba8UnormSrgb => 6,
        TextureFormat::Bgra8Unorm => 7,
        TextureFormat::Bgra8UnormSrgb => 8,
        TextureFormat::Rgba16Float => 9,
        TextureFormat::Rgba32Float => 10,
        TextureFormat::Depth24Plus => 11,
        TextureFormat::Depth24PlusStencil8 => 12,
        TextureFormat::Depth32Float => 13,
    }
}

fn texture_dimension_tag(dimension: TextureDimension) -> u8 {
    match dimension {
        TextureDimension::D1 => 0,
        TextureDimension::D2 => 1,
        TextureDimension::D2Array => 2,
        TextureDimension::D3 => 3,
        TextureDimension::Cube => 4,
    }
}

fn texture_residency_tag(residency: TextureResidency) -> u8 {
    match residency {
        TextureResidency::Dense => 0,
        TextureResidency::SparseReserved => 1,
    }
}

#[cfg(test)]
mod tests {
    use crate::graphics::backend::RenderBackend;
    use crate::rhi::{BufferUsage, TextureUsage};

    use super::*;

    #[test]
    fn transient_resource_pool_reuses_entries_across_frames() {
        let backend = RenderBackend::new_offscreen().unwrap();
        let desc = TextureDesc::new(
            "pooled-color",
            32,
            32,
            TextureFormat::Rgba8Unorm,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
        );
        let mut pool = TransientResourcePool::default();

        pool.begin_frame();
        let first = pool.acquire_texture(&backend.device, &desc);
        pool.release_texture(desc.clone(), first);
        pool.end_frame();
        assert_eq!(pool.last_frame_report().texture_created_count, 1);
        assert_eq!(pool.last_frame_report().texture_reused_count, 0);

        pool.begin_frame();
        let second = pool.acquire_texture(&backend.device, &desc);
        pool.release_texture(desc, second);
        pool.end_frame();
        assert_eq!(pool.last_frame_report().texture_created_count, 0);
        assert_eq!(pool.last_frame_report().texture_reused_count, 1);
        assert_eq!(pool.last_frame_report().texture_pool_entry_count, 1);
    }

    #[test]
    fn transient_resource_pool_evicts_stale_entries_after_keep_frames() {
        let backend = RenderBackend::new_offscreen().unwrap();
        let desc = BufferDesc::new(
            "pooled-buffer",
            64,
            BufferUsage::STORAGE | BufferUsage::COPY_DST,
        );
        let mut pool = TransientResourcePool::default();

        pool.begin_frame();
        let buffer = pool.acquire_buffer(&backend.device, &desc);
        pool.release_buffer(desc, buffer);
        pool.end_frame();

        for _ in 0..TRANSIENT_RESOURCE_POOL_KEEP_FRAMES {
            pool.begin_frame();
            pool.end_frame();
        }

        assert_eq!(pool.last_frame_report().evicted_buffer_count, 1);
        assert_eq!(pool.last_frame_report().buffer_pool_entry_count, 0);
    }
}
