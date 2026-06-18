use std::collections::BTreeMap;

use crate::core::framework::render::RenderGraphTransientPoolReport;
use crate::rhi::{BufferDesc, TextureDesc, TextureDimension, TextureFormat, TextureResidency};

use super::materialization::{create_wgpu_buffer, create_wgpu_texture};

pub(in crate::graphics::scene::scene_renderer) const TRANSIENT_RESOURCE_POOL_KEEP_FRAMES: u64 = 8;
const TRANSIENT_RESOURCE_POOL_MIB: u64 = 1024 * 1024;
pub(in crate::graphics::scene::scene_renderer) const TRANSIENT_RESOURCE_POOL_TEXTURE_BUDGET_BYTES:
    u64 = 256 * TRANSIENT_RESOURCE_POOL_MIB;
pub(in crate::graphics::scene::scene_renderer) const TRANSIENT_RESOURCE_POOL_BUFFER_BUDGET_BYTES:
    u64 = 64 * TRANSIENT_RESOURCE_POOL_MIB;

pub(in crate::graphics::scene::scene_renderer) struct TransientResourcePool {
    frame_index: u64,
    texture_budget_bytes: u64,
    buffer_budget_bytes: u64,
    frame_report: RenderGraphTransientPoolReport,
    last_frame_report: RenderGraphTransientPoolReport,
    textures: BTreeMap<TransientTextureKey, Vec<PooledTexture>>,
    buffers: BTreeMap<TransientBufferKey, Vec<PooledBuffer>>,
}

impl Default for TransientResourcePool {
    fn default() -> Self {
        Self {
            frame_index: 0,
            texture_budget_bytes: TRANSIENT_RESOURCE_POOL_TEXTURE_BUDGET_BYTES,
            buffer_budget_bytes: TRANSIENT_RESOURCE_POOL_BUFFER_BUDGET_BYTES,
            frame_report: RenderGraphTransientPoolReport::default(),
            last_frame_report: RenderGraphTransientPoolReport::default(),
            textures: BTreeMap::new(),
            buffers: BTreeMap::new(),
        }
    }
}

impl TransientResourcePool {
    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer) fn with_budgets(
        texture_budget_bytes: u64,
        buffer_budget_bytes: u64,
    ) -> Self {
        Self {
            texture_budget_bytes,
            buffer_budget_bytes,
            ..Self::default()
        }
    }

    pub fn begin_frame(&mut self) {
        self.frame_report = RenderGraphTransientPoolReport {
            frame_index: self.frame_index,
            texture_pool_entry_count: self.texture_entry_count(),
            buffer_pool_entry_count: self.buffer_entry_count(),
            texture_pool_retained_bytes: self.texture_pool_bytes(),
            buffer_pool_retained_bytes: self.buffer_pool_bytes(),
            texture_pool_budget_bytes: self.texture_budget_bytes,
            buffer_pool_budget_bytes: self.buffer_budget_bytes,
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
                byte_size: texture_desc_pool_size_bytes(&desc),
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
                byte_size: desc.size_bytes,
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
        self.frame_report.budget_evicted_texture_count =
            evict_textures_to_budget(&mut self.textures, self.texture_budget_bytes);
        self.frame_report.budget_evicted_buffer_count =
            evict_buffers_to_budget(&mut self.buffers, self.buffer_budget_bytes);
        self.frame_report.texture_pool_entry_count = self.texture_entry_count();
        self.frame_report.buffer_pool_entry_count = self.buffer_entry_count();
        self.frame_report.texture_pool_retained_bytes = self.texture_pool_bytes();
        self.frame_report.buffer_pool_retained_bytes = self.buffer_pool_bytes();
        self.frame_report.texture_pool_budget_bytes = self.texture_budget_bytes;
        self.frame_report.buffer_pool_budget_bytes = self.buffer_budget_bytes;
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

    fn texture_pool_bytes(&self) -> u64 {
        texture_pool_bytes(&self.textures)
    }

    fn buffer_pool_bytes(&self) -> u64 {
        buffer_pool_bytes(&self.buffers)
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
    byte_size: u64,
}

struct PooledBuffer {
    buffer: wgpu::Buffer,
    last_used_frame: u64,
    byte_size: u64,
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

fn evict_textures_to_budget(
    textures: &mut BTreeMap<TransientTextureKey, Vec<PooledTexture>>,
    budget_bytes: u64,
) -> usize {
    let mut evicted = 0;
    while texture_pool_bytes(textures) > budget_bytes {
        let Some((key, index)) = oldest_texture_entry(textures) else {
            break;
        };
        remove_texture_entry(textures, key, index);
        evicted += 1;
    }
    evicted
}

fn evict_buffers_to_budget(
    buffers: &mut BTreeMap<TransientBufferKey, Vec<PooledBuffer>>,
    budget_bytes: u64,
) -> usize {
    let mut evicted = 0;
    while buffer_pool_bytes(buffers) > budget_bytes {
        let Some((key, index)) = oldest_buffer_entry(buffers) else {
            break;
        };
        remove_buffer_entry(buffers, key, index);
        evicted += 1;
    }
    evicted
}

fn texture_pool_bytes(textures: &BTreeMap<TransientTextureKey, Vec<PooledTexture>>) -> u64 {
    textures
        .values()
        .flat_map(|entries| entries.iter())
        .fold(0, |total, entry| total.saturating_add(entry.byte_size))
}

fn buffer_pool_bytes(buffers: &BTreeMap<TransientBufferKey, Vec<PooledBuffer>>) -> u64 {
    buffers
        .values()
        .flat_map(|entries| entries.iter())
        .fold(0, |total, entry| total.saturating_add(entry.byte_size))
}

fn oldest_texture_entry(
    textures: &BTreeMap<TransientTextureKey, Vec<PooledTexture>>,
) -> Option<(TransientTextureKey, usize)> {
    textures
        .iter()
        .filter_map(|(key, entries)| {
            entries
                .iter()
                .enumerate()
                .min_by_key(|(_, entry)| entry.last_used_frame)
                .map(|(index, entry)| (*key, index, entry.last_used_frame))
        })
        .min_by_key(|(_, _, last_used_frame)| *last_used_frame)
        .map(|(key, index, _)| (key, index))
}

fn oldest_buffer_entry(
    buffers: &BTreeMap<TransientBufferKey, Vec<PooledBuffer>>,
) -> Option<(TransientBufferKey, usize)> {
    buffers
        .iter()
        .filter_map(|(key, entries)| {
            entries
                .iter()
                .enumerate()
                .min_by_key(|(_, entry)| entry.last_used_frame)
                .map(|(index, entry)| (*key, index, entry.last_used_frame))
        })
        .min_by_key(|(_, _, last_used_frame)| *last_used_frame)
        .map(|(key, index, _)| (key, index))
}

fn remove_texture_entry(
    textures: &mut BTreeMap<TransientTextureKey, Vec<PooledTexture>>,
    key: TransientTextureKey,
    index: usize,
) {
    let remove_bucket = if let Some(entries) = textures.get_mut(&key) {
        entries.swap_remove(index);
        entries.is_empty()
    } else {
        false
    };
    if remove_bucket {
        textures.remove(&key);
    }
}

fn remove_buffer_entry(
    buffers: &mut BTreeMap<TransientBufferKey, Vec<PooledBuffer>>,
    key: TransientBufferKey,
    index: usize,
) {
    let remove_bucket = if let Some(entries) = buffers.get_mut(&key) {
        entries.swap_remove(index);
        entries.is_empty()
    } else {
        false
    };
    if remove_bucket {
        buffers.remove(&key);
    }
}

fn texture_desc_pool_size_bytes(desc: &TextureDesc) -> u64 {
    desc.checked_storage_size_bytes().unwrap_or(u64::MAX)
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
    use crate::render_graph::{CompiledRenderGraph, QueueLane, RenderGraphBuilder};
    use crate::rhi::{BufferUsage, TextureUsage};

    use super::super::render_graph_execution_resources::RenderGraphExecutionResources;
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
        assert_eq!(pool.last_frame_report().texture_pool_retained_bytes, 4_096);
        assert_eq!(
            pool.last_frame_report().texture_pool_budget_bytes,
            TRANSIENT_RESOURCE_POOL_TEXTURE_BUDGET_BYTES
        );

        pool.begin_frame();
        let second = pool.acquire_texture(&backend.device, &desc);
        pool.release_texture(desc, second);
        pool.end_frame();
        assert_eq!(pool.last_frame_report().texture_created_count, 0);
        assert_eq!(pool.last_frame_report().texture_reused_count, 1);
        assert_eq!(pool.last_frame_report().texture_pool_entry_count, 1);
    }

    #[test]
    fn transient_resource_pool_evicts_oldest_entries_to_budget() {
        let backend = RenderBackend::new_offscreen().unwrap();
        let texture_desc = TextureDesc::new(
            "budgeted-color",
            32,
            32,
            TextureFormat::Rgba8Unorm,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
        );
        let buffer_desc = BufferDesc::new(
            "budgeted-buffer",
            64,
            BufferUsage::STORAGE | BufferUsage::COPY_DST,
        );
        let mut pool = TransientResourcePool::with_budgets(4_096, 64);

        pool.begin_frame();
        let first_texture = pool.acquire_texture(&backend.device, &texture_desc);
        let second_texture = pool.acquire_texture(&backend.device, &texture_desc);
        pool.release_texture(texture_desc.clone(), first_texture);
        pool.release_texture(texture_desc, second_texture);
        let first_buffer = pool.acquire_buffer(&backend.device, &buffer_desc);
        let second_buffer = pool.acquire_buffer(&backend.device, &buffer_desc);
        pool.release_buffer(buffer_desc.clone(), first_buffer);
        pool.release_buffer(buffer_desc, second_buffer);
        pool.end_frame();

        let report = pool.last_frame_report();
        assert_eq!(report.texture_created_count, 2);
        assert_eq!(report.buffer_created_count, 2);
        assert_eq!(report.budget_evicted_texture_count, 1);
        assert_eq!(report.budget_evicted_buffer_count, 1);
        assert_eq!(report.evicted_texture_count, 0);
        assert_eq!(report.evicted_buffer_count, 0);
        assert_eq!(report.texture_pool_entry_count, 1);
        assert_eq!(report.buffer_pool_entry_count, 1);
        assert_eq!(report.texture_pool_retained_bytes, 4_096);
        assert_eq!(report.buffer_pool_retained_bytes, 64);
        assert_eq!(report.texture_pool_budget_bytes, 4_096);
        assert_eq!(report.buffer_pool_budget_bytes, 64);
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
        assert_eq!(pool.last_frame_report().buffer_pool_retained_bytes, 0);
    }

    #[test]
    fn render_post_dynamic_resolution_scale_swap_releases_pool() {
        let backend = RenderBackend::new_offscreen().unwrap();
        let mut pool = TransientResourcePool::default();
        let half_resolution = dynamic_resolution_frame_graph("render-scale-0-5", 160, 120);
        let full_resolution = dynamic_resolution_frame_graph("render-scale-1-0", 320, 240);

        let first_half = materialize_graph_frame(&backend.device, &mut pool, &half_resolution);
        assert_eq!(first_half.texture_created_count, 1);
        assert_eq!(first_half.texture_reused_count, 0);
        assert_eq!(first_half.texture_pool_entry_count, 1);

        let full = materialize_graph_frame(&backend.device, &mut pool, &full_resolution);
        assert_eq!(full.texture_created_count, 1);
        assert_eq!(full.texture_reused_count, 0);
        assert_eq!(
            full.texture_pool_entry_count, 2,
            "switching from render_scale 0.5 to 1.0 should retain only the two live scale buckets"
        );

        let second_half = materialize_graph_frame(&backend.device, &mut pool, &half_resolution);
        assert_eq!(
            second_half.texture_created_count, 0,
            "returning to render_scale 0.5 must reuse the compatible half-size backing"
        );
        assert_eq!(second_half.texture_reused_count, 1);
        assert_eq!(
            second_half.texture_pool_entry_count, 2,
            "scale toggling must not grow the pool beyond the distinct descriptor buckets"
        );
    }

    fn dynamic_resolution_frame_graph(label: &str, width: u32, height: u32) -> CompiledRenderGraph {
        let mut builder = RenderGraphBuilder::new(label);
        let scene_color = builder.create_texture(TextureDesc::new(
            "scene-color",
            width,
            height,
            TextureFormat::Rgba8UnormSrgb,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
        ));
        let output = builder.import_external_resource("viewport-output");
        let write = builder.add_pass(format!("{label}-write"), QueueLane::Graphics);
        let present = builder.add_pass(format!("{label}-present"), QueueLane::Graphics);
        builder.write_texture(write, scene_color).unwrap();
        builder.read_texture(present, scene_color).unwrap();
        builder.write_external(present, output).unwrap();
        builder.add_dependency(write, present).unwrap();
        builder.compile().unwrap()
    }

    fn materialize_graph_frame(
        device: &wgpu::Device,
        pool: &mut TransientResourcePool,
        graph: &CompiledRenderGraph,
    ) -> RenderGraphTransientPoolReport {
        let mut resources = RenderGraphExecutionResources::new();

        pool.begin_frame();
        resources
            .materialize_transient_resources_with_pool(device, graph, pool)
            .unwrap();
        assert_eq!(
            resources.resource_report().owned_texture_count,
            1,
            "each dynamic-resolution graph frame should need one concrete scene-color backing"
        );
        resources.release_transient_backings_into_pool(pool);
        pool.end_frame();

        pool.last_frame_report()
    }
}
