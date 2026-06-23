use std::collections::{HashMap, HashSet};

use crate::rhi::{
    RhiError, UiSurfaceCommandKind, UiSurfaceDescriptor, UiSurfaceDrawList, UiSurfacePresentStats,
    UiSurfacePresenter,
};

mod batching;
mod geometry;
mod pipeline;
mod render_pass;
mod retained_cache;
mod surface_setup;
mod text;

use batching::{batch_draw_plan, BatchDrawPlanStats, DrawOp};
use geometry::command_effective_rect;
use pipeline::{
    create_image_bind_group_layout, create_image_pipeline, create_image_sampler,
    create_solid_pipeline,
};
use render_pass::{record_draw_ops_to_view, TargetLoad, WgpuUiDrawBuffers};
use retained_cache::WgpuRetainedSurfaceCache;
use surface_setup::{configure_surface, create_surface, instance_descriptor, request_device};
use text::WgpuUiTextRenderer;

// Editor image bytes are byte-space UI colors; keep upload textures out of sRGB
// so sampling them into the direct swapchain path stays byte-parity friendly.
const UI_IMAGE_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
const MAX_UI_IMAGE_CACHE_ENTRIES: usize = 256;

#[cfg(test)]
use surface_setup::{choose_alpha_mode, choose_surface_format};

pub struct WgpuUiSurfacePresenter {
    descriptor: UiSurfaceDescriptor,
    backend: WgpuUiSurfaceBackend,
    last_stats: UiSurfacePresentStats,
    presented_frame_count: u64,
}

enum WgpuUiSurfaceBackend {
    Headless,
    Native(Box<WgpuUiSurfaceRenderer>),
}

impl WgpuUiSurfacePresenter {
    pub fn new(descriptor: UiSurfaceDescriptor) -> Result<Self, RhiError> {
        descriptor.validate()?;
        let backend = if descriptor.target.is_some() {
            WgpuUiSurfaceBackend::Native(Box::new(WgpuUiSurfaceRenderer::new(descriptor)?))
        } else {
            WgpuUiSurfaceBackend::Headless
        };
        Ok(Self {
            descriptor,
            backend,
            last_stats: UiSurfacePresentStats {
                surface_size: descriptor.clamped_size(),
                ..UiSurfacePresentStats::default()
            },
            presented_frame_count: 0,
        })
    }

    pub fn new_headless(width: u32, height: u32) -> Self {
        Self::new(UiSurfaceDescriptor::headless(
            "wgpu-headless-ui-surface",
            width.max(1),
            height.max(1),
        ))
        .expect("headless descriptor is clamped to a valid size")
    }

    pub fn descriptor(&self) -> UiSurfaceDescriptor {
        self.descriptor
    }

    pub fn backend_name(&self) -> &'static str {
        match self.backend {
            WgpuUiSurfaceBackend::Headless => "wgpu-ui-surface-headless",
            WgpuUiSurfaceBackend::Native(_) => "wgpu-ui-surface",
        }
    }

    #[cfg(feature = "platform-winit")]
    pub fn descriptor_from_winit_window(
        label: &'static str,
        window: &dyn winit::window::Window,
    ) -> Result<UiSurfaceDescriptor, RhiError> {
        UiSurfaceDescriptor::from_winit_window(label, window)
    }
}

impl UiSurfacePresenter for WgpuUiSurfacePresenter {
    fn resize(&mut self, width: u32, height: u32) -> Result<(), RhiError> {
        self.descriptor.width = width.max(1);
        self.descriptor.height = height.max(1);
        if let WgpuUiSurfaceBackend::Native(renderer) = &mut self.backend {
            renderer.resize(self.descriptor.clamped_size())?;
        }
        self.last_stats.surface_size = self.descriptor.clamped_size();
        Ok(())
    }

    fn present(
        &mut self,
        draw_list: &UiSurfaceDrawList,
    ) -> Result<UiSurfacePresentStats, RhiError> {
        if draw_list.surface_size != self.descriptor.clamped_size() {
            self.resize(draw_list.surface_size.0, draw_list.surface_size.1)?;
        }
        let presentation = match &mut self.backend {
            WgpuUiSurfaceBackend::Native(renderer) => renderer.present(draw_list)?,
            WgpuUiSurfaceBackend::Headless => {
                let draw_plan = batch_draw_plan(draw_list);
                WgpuUiSurfacePresentation {
                    draw_list_stats: draw_list.stats(),
                    batch_stats: draw_plan.stats,
                }
            }
        };

        let mut stats = presentation.draw_list_stats;
        stats.draw_calls = presentation.batch_stats.draw_calls;
        stats.visible_draw_item_count = presentation.batch_stats.visible_draw_item_count;
        stats.batch_layer_count = presentation.batch_stats.batch_layer_count;
        stats.batch_dependency_count = presentation.batch_stats.batch_dependency_count;
        self.presented_frame_count = self.presented_frame_count.saturating_add(1);
        stats.presented_frame_count = self.presented_frame_count;
        self.last_stats = stats.clone();
        Ok(stats)
    }

    fn last_present_stats(&self) -> UiSurfacePresentStats {
        self.last_stats.clone()
    }
}

struct WgpuUiSurfacePresentation {
    draw_list_stats: UiSurfacePresentStats,
    batch_stats: BatchDrawPlanStats,
}

struct WgpuUiSurfaceRenderer {
    _instance: wgpu::Instance,
    _adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    solid_pipeline: wgpu::RenderPipeline,
    image_pipeline: wgpu::RenderPipeline,
    image_bind_group_layout: wgpu::BindGroupLayout,
    image_sampler: wgpu::Sampler,
    retained_cache: WgpuRetainedSurfaceCache,
    image_cache: HashMap<String, WgpuUiImageResource>,
    text: WgpuUiTextRenderer,
    present_index: u64,
}

impl WgpuUiSurfaceRenderer {
    fn new(descriptor: UiSurfaceDescriptor) -> Result<Self, RhiError> {
        let target = descriptor
            .target
            .ok_or_else(|| RhiError::SurfaceUnavailable("missing native surface target".into()))?;
        let instance = wgpu::Instance::new(instance_descriptor());
        let surface = create_surface(&instance, target)?;
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .map_err(|_| RhiError::SurfaceUnavailable("no compatible adapter found".into()))?;
        let (device, queue) = request_device(&adapter)?;
        let size = descriptor.clamped_size();
        let config = configure_surface(&surface, &adapter, &device, size)?;
        let solid_pipeline = create_solid_pipeline(&device, config.format);
        let image_bind_group_layout = create_image_bind_group_layout(&device);
        let image_sampler = create_image_sampler(&device);
        let image_pipeline =
            create_image_pipeline(&device, config.format, &image_bind_group_layout);
        let retained_cache = WgpuRetainedSurfaceCache::new(
            &device,
            config.format,
            size,
            &image_bind_group_layout,
            &image_sampler,
        );
        let text = WgpuUiTextRenderer::new(&device, &queue, config.format);

        Ok(Self {
            _instance: instance,
            _adapter: adapter,
            device,
            queue,
            surface,
            config,
            solid_pipeline,
            image_pipeline,
            image_bind_group_layout,
            image_sampler,
            retained_cache,
            image_cache: HashMap::new(),
            text,
            present_index: 0,
        })
    }

    fn resize(&mut self, size: (u32, u32)) -> Result<(), RhiError> {
        let size = (size.0.max(1), size.1.max(1));
        self.config.width = size.0;
        self.config.height = size.1;
        self.surface.configure(&self.device, &self.config);
        self.retained_cache.resize(
            &self.device,
            self.config.format,
            size,
            &self.image_bind_group_layout,
            &self.image_sampler,
        );
        Ok(())
    }

    fn present(
        &mut self,
        draw_list: &UiSurfaceDrawList,
    ) -> Result<WgpuUiSurfacePresentation, RhiError> {
        self.resize_if_needed(draw_list.surface_size)?;
        self.present_index = self.present_index.saturating_add(1);
        let cache_ready = self
            .retained_cache
            .matches(self.config.format, draw_list.surface_size)
            && self.retained_cache.initialized();
        let mode = surface_render_mode(draw_list, cache_ready);
        let render_draw_list;
        let draw_list = match mode {
            SurfaceRenderMode::FullRedraw => {
                render_draw_list = full_redraw_draw_list(draw_list);
                &render_draw_list
            }
            SurfaceRenderMode::DamagePatch => draw_list,
        };
        let draw_plan = batch_draw_plan(draw_list);
        self.prepare_image_resources(draw_list);
        self.text.prepare(
            &self.device,
            &self.queue,
            draw_list.surface_size,
            draw_list,
            &draw_plan.ops,
        );
        self.render_draw_list_to_surface(draw_list, &draw_plan.ops, mode)?;
        self.prune_image_cache();
        let mut batch_stats = draw_plan.stats;
        if mode == SurfaceRenderMode::DamagePatch {
            batch_stats.draw_calls = batch_stats.draw_calls.saturating_add(1);
        }
        Ok(WgpuUiSurfacePresentation {
            draw_list_stats: draw_list.stats(),
            batch_stats,
        })
    }

    fn resize_if_needed(&mut self, size: (u32, u32)) -> Result<(), RhiError> {
        if size != (self.config.width, self.config.height) {
            self.resize(size)?;
        }
        Ok(())
    }

    fn prepare_image_resources(&mut self, draw_list: &UiSurfaceDrawList) {
        let mut uploaded_resource_keys = HashSet::new();
        for command in &draw_list.commands {
            let UiSurfaceCommandKind::Image { payload } = &command.kind else {
                continue;
            };
            if command_effective_rect(command, draw_list).is_none() {
                continue;
            }
            if payload.resource_key.is_empty() {
                continue;
            }
            let Some(rgba) = payload.rgba.as_deref() else {
                if let Some(resource) = self.image_cache.get_mut(&payload.resource_key) {
                    resource.last_touched_present = self.present_index;
                }
                continue;
            };
            if payload.width == 0 || payload.height == 0 {
                continue;
            }
            let expected_len = payload.width as usize * payload.height as usize * 4;
            if rgba.len() < expected_len {
                continue;
            }
            let cache_key = payload.resource_key.clone();
            if !uploaded_resource_keys.insert(cache_key.clone()) {
                if let Some(resource) = self.image_cache.get_mut(&cache_key) {
                    resource.last_touched_present = self.present_index;
                }
                continue;
            }
            let replace = self
                .image_cache
                .get(&cache_key)
                .map(|resource| resource.size != (payload.width, payload.height))
                .unwrap_or(true);
            if replace {
                let resource = WgpuUiImageResource::new(
                    &self.device,
                    &self.image_bind_group_layout,
                    &self.image_sampler,
                    &cache_key,
                    (payload.width, payload.height),
                    self.present_index,
                );
                self.image_cache.insert(cache_key.clone(), resource);
            }
            if let Some(resource) = self.image_cache.get_mut(&cache_key) {
                resource.last_touched_present = self.present_index;
                self.queue.write_texture(
                    resource.texture.as_image_copy(),
                    &rgba[..expected_len],
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(payload.width * 4),
                        rows_per_image: Some(payload.height),
                    },
                    wgpu::Extent3d {
                        width: payload.width,
                        height: payload.height,
                        depth_or_array_layers: 1,
                    },
                );
            }
        }
    }

    fn prune_image_cache(&mut self) {
        let keys_to_prune = image_cache_keys_to_prune(
            self.image_cache
                .iter()
                .map(|(key, resource)| (key.as_str(), resource.last_touched_present)),
            MAX_UI_IMAGE_CACHE_ENTRIES,
        );
        for key in keys_to_prune {
            self.image_cache.remove(&key);
        }
    }

    fn render_draw_list_to_surface(
        &mut self,
        draw_list: &UiSurfaceDrawList,
        draw_ops: &[DrawOp],
        mode: SurfaceRenderMode,
    ) -> Result<(), RhiError> {
        let surface_texture = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(surface_texture)
            | wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => surface_texture,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.config);
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err(RhiError::SurfaceUnavailable(
                    "surface validation error".to_string(),
                ));
            }
        };
        let target_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let buffers = WgpuUiDrawBuffers::new(&self.device, draw_ops);
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("zircon-ui-surface-encoder"),
            });

        match mode {
            SurfaceRenderMode::FullRedraw => {
                record_draw_ops_to_view(
                    &mut encoder,
                    &target_view,
                    TargetLoad::ClearBlack,
                    draw_list.surface_size,
                    draw_ops,
                    &buffers,
                    &self.solid_pipeline,
                    &self.image_pipeline,
                    &self.image_cache,
                    &mut self.text,
                );
                record_draw_ops_to_view(
                    &mut encoder,
                    self.retained_cache.view(),
                    TargetLoad::ClearBlack,
                    draw_list.surface_size,
                    draw_ops,
                    &buffers,
                    &self.solid_pipeline,
                    &self.image_pipeline,
                    &self.image_cache,
                    &mut self.text,
                );
                self.retained_cache.mark_initialized();
            }
            SurfaceRenderMode::DamagePatch => {
                self.retained_cache.record_restore(
                    &mut encoder,
                    &self.image_pipeline,
                    &target_view,
                    draw_list.surface_size,
                );
                record_draw_ops_to_view(
                    &mut encoder,
                    &target_view,
                    TargetLoad::Load,
                    draw_list.surface_size,
                    draw_ops,
                    &buffers,
                    &self.solid_pipeline,
                    &self.image_pipeline,
                    &self.image_cache,
                    &mut self.text,
                );
                record_draw_ops_to_view(
                    &mut encoder,
                    self.retained_cache.view(),
                    TargetLoad::Load,
                    draw_list.surface_size,
                    draw_ops,
                    &buffers,
                    &self.solid_pipeline,
                    &self.image_pipeline,
                    &self.image_cache,
                    &mut self.text,
                );
                self.retained_cache.mark_initialized();
            }
        }

        self.queue.submit(Some(encoder.finish()));
        surface_texture.present();
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SurfaceRenderMode {
    FullRedraw,
    DamagePatch,
}

fn surface_render_mode(draw_list: &UiSurfaceDrawList, cache_ready: bool) -> SurfaceRenderMode {
    if draw_list.damage.is_some() && cache_ready {
        SurfaceRenderMode::DamagePatch
    } else {
        SurfaceRenderMode::FullRedraw
    }
}

fn full_redraw_draw_list(draw_list: &UiSurfaceDrawList) -> UiSurfaceDrawList {
    let mut draw_list = draw_list.clone();
    draw_list.damage = None;
    draw_list
}

struct WgpuUiImageResource {
    texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
    size: (u32, u32),
    last_touched_present: u64,
}

impl WgpuUiImageResource {
    fn new(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
        key: &str,
        size: (u32, u32),
        last_touched_present: u64,
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-ui-image"),
            size: wgpu::Extent3d {
                width: size.0.max(1),
                height: size.1.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: UI_IMAGE_TEXTURE_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-ui-image-bind-group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        });
        let _ = key;
        Self {
            texture,
            bind_group,
            size,
            last_touched_present,
        }
    }
}

fn image_cache_keys_to_prune<'a>(
    entries: impl Iterator<Item = (&'a str, u64)>,
    max_entries: usize,
) -> Vec<String> {
    let mut entries = entries
        .map(|(key, last_touched_present)| (last_touched_present, key))
        .collect::<Vec<_>>();
    if entries.len() <= max_entries {
        return Vec::new();
    }

    let prune_count = entries.len() - max_entries;
    entries.sort_unstable();
    entries
        .into_iter()
        .take(prune_count)
        .map(|(_, key)| key.to_string())
        .collect()
}

#[cfg(test)]
mod tests;
