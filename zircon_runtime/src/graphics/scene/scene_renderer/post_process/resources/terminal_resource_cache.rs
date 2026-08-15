use std::sync::{Arc, Mutex, MutexGuard};

use crate::core::math::UVec2;
use crate::graphics::scene::scene_renderer::post_process::SMAA_STAGE_FORMAT;
use crate::graphics::types::ViewportRenderRegion;

use wgpu::util::DeviceExt;

use super::super::post_process_params::TerminalRegionParams;

const MAX_CACHED_PHYSICAL_TERMINAL_REGIONS: usize = 16;
const MAX_CACHED_SMAA_EXTENTS: usize = 1;

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

struct BoundedResourceCache<K, V> {
    capacity: usize,
    entries: Vec<(K, Arc<V>)>,
}

impl<K, V> BoundedResourceCache<K, V>
where
    K: PartialEq,
{
    fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            entries: Vec::with_capacity(capacity.max(1)),
        }
    }

    fn get_or_insert_with(&mut self, key: K, create: impl FnOnce() -> V) -> Arc<V> {
        if let Some(index) = self
            .entries
            .iter()
            .position(|(candidate, _)| candidate == &key)
        {
            let entry = self.entries.remove(index);
            let resource = entry.1.clone();
            self.entries.push(entry);
            return resource;
        }
        if self.entries.len() == self.capacity {
            self.entries.remove(0);
        }
        let resource = Arc::new(create());
        self.entries.push((key, resource.clone()));
        resource
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::sync::Arc;

    use crate::core::math::UVec2;
    use crate::graphics::types::ViewportRenderRegion;

    use super::{BoundedResourceCache, TerminalPostProcessResourceCache};

    #[test]
    fn bounded_resource_cache_reuses_a_matching_resource_without_recreating_it() {
        let creates = Cell::new(0_u32);
        let mut cache = BoundedResourceCache::new(2);

        let first = cache.get_or_insert_with(64_u32, || {
            creates.set(creates.get() + 1);
            1_u32
        });
        let second = cache.get_or_insert_with(64_u32, || {
            creates.set(creates.get() + 1);
            2_u32
        });

        assert!(Arc::ptr_eq(&first, &second));
        assert_eq!(creates.get(), 1);
    }

    #[test]
    fn bounded_resource_cache_evicts_the_oldest_entry_at_its_fixed_capacity() {
        let mut cache = BoundedResourceCache::new(1);
        let first = cache.get_or_insert_with(16_u32, || 1_u32);
        let replacement = cache.get_or_insert_with(32_u32, || 2_u32);
        let rebuilt = cache.get_or_insert_with(16_u32, || 3_u32);

        assert_eq!(*replacement, 2);
        assert_eq!(*rebuilt, 3);
        assert!(!Arc::ptr_eq(&first, &rebuilt));
    }

    #[test]
    fn terminal_effect_executors_delegate_persistent_resources_to_the_cache_owner() {
        let fxaa = include_str!("execute_fxaa/mod.rs");
        let output_transfer = include_str!("execute_output_transfer/mod.rs");
        let smaa = include_str!("execute_smaa/mod.rs");

        for (label, source) in [("FXAA", fxaa), ("output transfer", output_transfer)] {
            assert!(
                source.contains("physical_terminal_region_params_buffer(device, render_region)"),
                "{label} must resolve terminal uniforms through the persistent owner"
            );
            assert!(
                !source.contains("create_physical_terminal_region_params_buffer"),
                "{label} must not create a physical terminal uniform every frame"
            );
        }
        assert!(
            smaa.contains("smaa_stage_textures(device, viewport_size)"),
            "SMAA must resolve edge/blend backing textures through the persistent owner"
        );
        assert!(
            !smaa.contains("create_smaa_stage_texture"),
            "SMAA must not create edge/blend backing textures every frame"
        );
    }

    #[test]
    fn scene_post_process_resources_constructs_the_terminal_cache_at_the_resources_root() {
        let resources = include_str!("../scene_post_process_resources/mod.rs");
        let constructor = include_str!("construct/construct/construct.rs");
        let compact_resources = resources.split_whitespace().collect::<String>();
        let compact_constructor = constructor.split_whitespace().collect::<String>();

        assert!(
            compact_resources.contains("terminal_resource_cache:TerminalPostProcessResourceCache,"),
            "the persistent cache belongs to the post-process resource owner"
        );
        assert!(
            compact_constructor.contains(
                "usesuper::super::super::terminal_resource_cache::TerminalPostProcessResourceCache;"
            ),
            "the nested constructor must import the cache from the resources root"
        );
        assert!(
            compact_constructor
                .contains("terminal_resource_cache:TerminalPostProcessResourceCache::new(),"),
            "the post-process resource owner must initialize the cache once"
        );
    }

    #[test]
    fn terminal_resource_cache_reuses_wgpu_backing_until_the_extent_changes() {
        let Some(device) = offscreen_test_device() else {
            eprintln!("skipping terminal resource cache GPU test: no WGPU adapter is available");
            return;
        };
        let cache = TerminalPostProcessResourceCache::new();
        let region = ViewportRenderRegion::full_target(UVec2::new(64, 48));

        let first_uniform = cache.physical_terminal_region_params_buffer(&device, region);
        let warm_uniform = cache.physical_terminal_region_params_buffer(&device, region);
        let first_stages = cache.smaa_stage_textures(&device, UVec2::new(64, 48));
        let warm_stages = cache.smaa_stage_textures(&device, UVec2::new(64, 48));
        let resized_stages = cache.smaa_stage_textures(&device, UVec2::new(96, 64));

        assert!(Arc::ptr_eq(&first_uniform, &warm_uniform));
        assert!(Arc::ptr_eq(&first_stages, &warm_stages));
        assert!(!Arc::ptr_eq(&first_stages, &resized_stages));
    }

    fn offscreen_test_device() -> Option<wgpu::Device> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: None,
            force_fallback_adapter: false,
        }))
        .ok()?;
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("zircon-terminal-resource-cache-test-device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::Off,
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
        }))
        .ok()
        .map(|(device, _queue)| device)
    }
}
