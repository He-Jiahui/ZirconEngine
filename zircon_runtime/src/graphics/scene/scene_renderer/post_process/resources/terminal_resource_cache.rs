use std::sync::{Arc, Mutex, MutexGuard};

use crate::core::math::UVec2;
use crate::graphics::scene::scene_renderer::post_process::SMAA_STAGE_FORMAT;
use crate::graphics::types::ViewportRenderRegion;

use wgpu::util::DeviceExt;

use super::super::post_process_params::TerminalRegionParams;

const MAX_CACHED_PHYSICAL_TERMINAL_REGIONS: usize = 16;
const MAX_CACHED_SMAA_EXTENTS: usize = 1;

/// 同一渲染器设备的终端效果资源缓存，避免每帧创建不可变区域参数和 SMAA 中间纹理。
/// 所有请求须来自创建该后处理资源的设备；返回的 Arc 可让被淘汰资源继续供已录制命令使用。
pub(in crate::graphics::scene::scene_renderer::post_process) struct TerminalPostProcessResourceCache
{
    state: Mutex<TerminalPostProcessResourceCacheState>,
}

struct TerminalPostProcessResourceCacheState {
    local_terminal_region_params: Option<Arc<wgpu::Buffer>>,
    physical_terminal_region_params: BoundedResourceCache<[u32; 2], wgpu::Buffer>,
    smaa_stage_textures: BoundedResourceCache<[u32; 2], SmaaStageTextures>,
}

impl TerminalPostProcessResourceCache {
    pub(in crate::graphics::scene::scene_renderer::post_process) fn new() -> Self {
        Self {
            state: Mutex::new(TerminalPostProcessResourceCacheState {
                local_terminal_region_params: None,
                physical_terminal_region_params: BoundedResourceCache::new(
                    MAX_CACHED_PHYSICAL_TERMINAL_REGIONS,
                ),
                smaa_stage_textures: BoundedResourceCache::new(MAX_CACHED_SMAA_EXTENTS),
            }),
        }
    }

    pub(in crate::graphics::scene::scene_renderer::post_process) fn local_terminal_region_params_buffer(
        &self,
        device: &wgpu::Device,
    ) -> Arc<wgpu::Buffer> {
        let state = &mut *self.lock_state();
        state
            .local_terminal_region_params
            .get_or_insert_with(|| Arc::new(create_terminal_region_params_buffer(device, [0, 0])))
            .clone()
    }

    pub(in crate::graphics::scene::scene_renderer::post_process) fn physical_terminal_region_params_buffer(
        &self,
        device: &wgpu::Device,
        render_region: ViewportRenderRegion,
    ) -> Arc<wgpu::Buffer> {
        let origin = render_region.physical_origin();
        let state = &mut *self.lock_state();
        state
            .physical_terminal_region_params
            .get_or_insert_with(origin, || {
                create_terminal_region_params_buffer(device, origin)
            })
    }

    pub(in crate::graphics::scene::scene_renderer::post_process) fn smaa_stage_textures(
        &self,
        device: &wgpu::Device,
        viewport_size: UVec2,
    ) -> Arc<SmaaStageTextures> {
        let extent = [viewport_size.x.max(1), viewport_size.y.max(1)];
        let state = &mut *self.lock_state();
        state
            .smaa_stage_textures
            .get_or_insert_with(extent, || SmaaStageTextures::new(device, extent))
    }

    fn lock_state(&self) -> MutexGuard<'_, TerminalPostProcessResourceCacheState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

pub(in crate::graphics::scene::scene_renderer::post_process) struct SmaaStageTextures {
    _edge_texture: wgpu::Texture,
    edge_view: wgpu::TextureView,
    _blend_texture: wgpu::Texture,
    blend_view: wgpu::TextureView,
}

impl SmaaStageTextures {
    fn new(device: &wgpu::Device, extent: [u32; 2]) -> Self {
        let edge_texture = create_smaa_stage_texture(device, extent, "zircon-smaa-edges");
        let blend_texture = create_smaa_stage_texture(device, extent, "zircon-smaa-blend");
        Self {
            edge_view: edge_texture.create_view(&wgpu::TextureViewDescriptor::default()),
            blend_view: blend_texture.create_view(&wgpu::TextureViewDescriptor::default()),
            _edge_texture: edge_texture,
            _blend_texture: blend_texture,
        }
    }

    pub(in crate::graphics::scene::scene_renderer::post_process) fn edge_view(
        &self,
    ) -> &wgpu::TextureView {
        &self.edge_view
    }

    pub(in crate::graphics::scene::scene_renderer::post_process) fn blend_view(
        &self,
    ) -> &wgpu::TextureView {
        &self.blend_view
    }
}

fn create_terminal_region_params_buffer(device: &wgpu::Device, origin: [u32; 2]) -> wgpu::Buffer {
    let params = TerminalRegionParams {
        viewport_origin: [origin[0], origin[1], 0, 0],
    };
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("zircon-terminal-region-params"),
        contents: bytemuck::bytes_of(&params),
        usage: wgpu::BufferUsages::UNIFORM,
    })
}

fn create_smaa_stage_texture(
    device: &wgpu::Device,
    extent: [u32; 2],
    label: &'static str,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: extent[0],
            height: extent[1],
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: SMAA_STAGE_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    })
}

// 小容量缓存使用访问顺序淘汰；淘汰只释放缓存所有权，已返回的 Arc 保留在途资源。
struct BoundedResourceCache<K, V> {
    capacity: usize,
    entries: Vec<BoundedResourceCacheEntry<K, V>>,
    access_epoch: u64,
}

struct BoundedResourceCacheEntry<K, V> {
    key: K,
    resource: Arc<V>,
    last_used: u64,
}

impl<K, V> BoundedResourceCache<K, V>
where
    K: PartialEq,
{
    fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            entries: Vec::with_capacity(capacity.max(1)),
            access_epoch: 0,
        }
    }

    fn get_or_insert_with(&mut self, key: K, create: impl FnOnce() -> V) -> Arc<V> {
        self.access_epoch += 1;
        if let Some(entry) = self.entries.iter_mut().find(|entry| entry.key == key) {
            entry.last_used = self.access_epoch;
            return entry.resource.clone();
        }
        let resource = Arc::new(create());
        let entry = BoundedResourceCacheEntry {
            key,
            resource: resource.clone(),
            last_used: self.access_epoch,
        };
        if self.entries.len() == self.capacity {
            let oldest_index = self
                .entries
                .iter()
                .enumerate()
                .min_by_key(|(_, entry)| entry.last_used)
                .map(|(index, _)| index)
                .expect("a full bounded cache has an oldest entry");
            self.entries[oldest_index] = entry;
        } else {
            self.entries.push(entry);
        }
        resource
    }
}

#[cfg(test)]
#[path = "tests/terminal_resource_cache.rs"]
mod tests;
