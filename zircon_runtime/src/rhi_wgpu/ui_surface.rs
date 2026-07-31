use std::collections::HashMap;

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

use batching::{BatchDrawPlan, BatchDrawPlanStats, CompiledUiBatchPlanCache, ImageUploadSource};
use pipeline::{
    create_image_bind_group_layout, create_image_pipeline, create_image_sampler,
    create_solid_pipeline,
};
use render_pass::{
    record_draw_ops_to_view, TargetLoad, WgpuUiDrawBufferCache, WgpuUiDrawBufferStats,
    WgpuUiRecordedDrawStats,
};
use retained_cache::WgpuRetainedSurfaceCache;
use surface_setup::{configure_surface, create_surface, instance_descriptor, request_device};
use text::{WgpuUiTextPrepareStats, WgpuUiTextRenderer};

// Editor image bytes are byte-space UI colors; keep upload textures out of sRGB
// so sampling them into the direct swapchain path stays byte-parity friendly.
const UI_IMAGE_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
const MAX_UI_IMAGE_CACHE_ENTRIES: usize = 256;
const MAX_UI_IMAGE_CACHE_BYTES: u64 = 64 * 1024 * 1024;

#[cfg(test)]
use surface_setup::{choose_alpha_mode, choose_surface_format, choose_surface_usage};

pub struct WgpuUiSurfacePresenter {
    descriptor: UiSurfaceDescriptor,
    backend: WgpuUiSurfaceBackend,
    last_stats: UiSurfacePresentStats,
    presented_frame_count: u64,
}

enum WgpuUiSurfaceBackend {
    Headless(CompiledUiBatchPlanCache),
    Native(Box<WgpuUiSurfaceRenderer>),
}

impl WgpuUiSurfacePresenter {
    pub fn new(descriptor: UiSurfaceDescriptor) -> Result<Self, RhiError> {
        descriptor.validate()?;
        let backend = if descriptor.target.is_some() {
            WgpuUiSurfaceBackend::Native(Box::new(WgpuUiSurfaceRenderer::new(descriptor)?))
        } else {
            WgpuUiSurfaceBackend::Headless(CompiledUiBatchPlanCache::default())
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
            WgpuUiSurfaceBackend::Headless(_) => "wgpu-ui-surface-headless",
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
            WgpuUiSurfaceBackend::Headless(compiled_batch_plan) => {
                let resolved_draw_plan = compiled_batch_plan.resolve(draw_list, false);
                let mut batch_stats = resolved_draw_plan.plan.stats;
                batch_stats.batch_plan_build_count = resolved_draw_plan.batch_plan_build_count;
                batch_stats.batch_plan_cache_hit_count =
                    resolved_draw_plan.batch_plan_cache_hit_count;
                WgpuUiSurfacePresentation {
                    draw_list_stats: resolved_draw_plan
                        .draw_list_stats
                        .unwrap_or_else(|| draw_list.stats()),
                    batch_stats,
                    text_stats: WgpuUiTextPrepareStats::default(),
                    image_resource_stats: None,
                    recorded_stats: None,
                }
            }
        };

        let mut stats = presentation.draw_list_stats;
        stats.compiled_draw_calls = presentation.batch_stats.draw_calls;
        stats.compiled_visible_draw_item_count = presentation.batch_stats.visible_draw_item_count;
        stats.compiled_solid_vertex_count = presentation.batch_stats.solid_vertex_count;
        stats.compiled_image_vertex_count = presentation.batch_stats.image_vertex_count;
        stats.compiled_batch_layer_count = presentation.batch_stats.batch_layer_count;
        stats.compiled_batch_dependency_count = presentation.batch_stats.batch_dependency_count;
        stats.compiled_batch_merge_count = presentation.batch_stats.batch_merge_count;
        stats.draw_calls = presentation.batch_stats.draw_calls;
        stats.render_pass_count = presentation.batch_stats.render_pass_count;
        stats.visible_draw_item_count = presentation.batch_stats.visible_draw_item_count;
        stats.batch_merge_count = presentation.batch_stats.batch_merge_count;
        stats.solid_vertex_count = presentation.batch_stats.solid_vertex_count;
        stats.image_vertex_count = presentation.batch_stats.image_vertex_count;
        stats.batch_layer_count = presentation.batch_stats.batch_layer_count;
        stats.batch_dependency_count = presentation.batch_stats.batch_dependency_count;
        stats.overlap_candidate_count = presentation.batch_stats.overlap_candidate_count;
        stats.batch_plan_build_count = presentation.batch_stats.batch_plan_build_count;
        stats.batch_plan_cache_hit_count = presentation.batch_stats.batch_plan_cache_hit_count;
        stats.vertex_buffer_create_count = presentation.batch_stats.vertex_buffer_create_count;
        stats.vertex_upload_bytes = presentation.batch_stats.vertex_upload_bytes;
        stats.retained_cache_copy_bytes = presentation.batch_stats.retained_cache_copy_bytes;
        if let Some(recorded) = presentation.recorded_stats {
            stats.draw_calls = recorded.draw_calls;
            stats.render_pass_count = recorded.render_pass_count;
            stats.visible_draw_item_count = recorded.visible_draw_item_count;
            stats.solid_vertex_count = recorded.solid_vertex_count;
            stats.image_vertex_count = recorded.image_vertex_count;
            stats.batch_layer_count = recorded.batch_layer_count;
            stats.batch_dependency_count = 0;
            stats.batch_merge_count = recorded
                .visible_draw_item_count
                .saturating_sub(recorded.draw_calls);
        }
        stats.text_shape_count = presentation.text_stats.text_shape_count;
        stats.text_renderer_build_count = presentation.text_stats.text_renderer_build_count;
        stats.text_renderer_cache_hit_count = presentation.text_stats.text_renderer_cache_hit_count;
        stats.text_prepare_failure_count = presentation.text_stats.text_prepare_failure_count;
        if let Some(image_resource_stats) = presentation.image_resource_stats {
            stats.image_upload_bytes = image_resource_stats.upload_bytes;
            stats.image_upload_write_count = image_resource_stats.upload_write_count;
            stats.image_cache_key_allocation_count =
                image_resource_stats.cache_key_allocation_count;
            stats.image_cache_prune_visit_count = image_resource_stats.cache_prune_visit_count;
            stats.image_cache_admission_reject_count =
                image_resource_stats.cache_admission_reject_count;
            stats.image_invalid_payload_count = image_resource_stats.invalid_payload_count;
            stats.image_cache_resident_bytes = image_resource_stats.cache_resident_bytes;
            stats.image_prepare_command_visit_count =
                image_resource_stats.prepare_command_visit_count;
            stats.image_prepare_cache_hit_count = image_resource_stats.prepare_cache_hit_count;
        }
        self.presented_frame_count = self.presented_frame_count.saturating_add(1);
        stats.presented_frame_count = self.presented_frame_count;
        self.last_stats = stats;
        Ok(stats)
    }

    fn last_present_stats(&self) -> UiSurfacePresentStats {
        self.last_stats
    }
}

struct WgpuUiSurfacePresentation {
    draw_list_stats: UiSurfacePresentStats,
    batch_stats: BatchDrawPlanStats,
    text_stats: WgpuUiTextPrepareStats,
    image_resource_stats: Option<WgpuUiImageResourceStats>,
    recorded_stats: Option<WgpuUiRecordedDrawStats>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct WgpuUiImageResourceStats {
    upload_bytes: u64,
    upload_write_count: u64,
    cache_key_allocation_count: u64,
    cache_prune_visit_count: u64,
    cache_admission_reject_count: u64,
    invalid_payload_count: u64,
    cache_resident_bytes: u64,
    prepare_command_visit_count: u64,
    prepare_cache_hit_count: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct WgpuUiSurfaceRenderStats {
    draw_buffers: WgpuUiDrawBufferStats,
    recorded: WgpuUiRecordedDrawStats,
    retained_cache_copy_bytes: u64,
}

impl WgpuUiSurfaceRenderStats {
    fn add_recorded(&mut self, recorded: WgpuUiRecordedDrawStats) {
        self.recorded.draw_calls = self.recorded.draw_calls.saturating_add(recorded.draw_calls);
        self.recorded.render_pass_count = self
            .recorded
            .render_pass_count
            .saturating_add(recorded.render_pass_count);
        self.recorded.visible_draw_item_count = self
            .recorded
            .visible_draw_item_count
            .saturating_add(recorded.visible_draw_item_count);
        self.recorded.solid_vertex_count = self
            .recorded
            .solid_vertex_count
            .saturating_add(recorded.solid_vertex_count);
        self.recorded.image_vertex_count = self
            .recorded
            .image_vertex_count
            .saturating_add(recorded.image_vertex_count);
        self.recorded.batch_layer_count = self
            .recorded
            .batch_layer_count
            .saturating_add(recorded.batch_layer_count);
    }
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
    retained_cache: Option<WgpuRetainedSurfaceCache>,
    image_cache: HashMap<String, WgpuUiImageResource>,
    image_cache_bytes: u64,
    text: WgpuUiTextRenderer,
    compiled_batch_plan: CompiledUiBatchPlanCache,
    compiled_draw_buffers: WgpuUiDrawBufferCache,
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
        let retained_cache = retained_cache_copy_supported(config.usage)
            .then(|| WgpuRetainedSurfaceCache::new(&device, config.format, size));
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
            image_cache_bytes: 0,
            text,
            compiled_batch_plan: CompiledUiBatchPlanCache::default(),
            compiled_draw_buffers: WgpuUiDrawBufferCache::default(),
            present_index: 0,
        })
    }

    fn resize(&mut self, size: (u32, u32)) -> Result<(), RhiError> {
        let size = (size.0.max(1), size.1.max(1));
        self.config.width = size.0;
        self.config.height = size.1;
        self.surface.configure(&self.device, &self.config);
        if let Some(retained_cache) = &mut self.retained_cache {
            retained_cache.resize(&self.device, self.config.format, size);
        }
        Ok(())
    }

    fn present(
        &mut self,
        draw_list: &UiSurfaceDrawList,
    ) -> Result<WgpuUiSurfacePresentation, RhiError> {
        self.resize_if_needed(draw_list.surface_size)?;
        self.present_index = self.present_index.saturating_add(1);
        let cache_ready = self.retained_cache.as_ref().is_some_and(|retained_cache| {
            retained_cache.matches(self.config.format, draw_list.surface_size)
                && retained_cache.initialized()
        });
        let mode = surface_render_mode(draw_list, cache_ready);
        let damage = render_damage(draw_list, mode);
        let resolved_draw_plan = self
            .compiled_batch_plan
            .resolve(draw_list, mode == SurfaceRenderMode::FullRedraw);
        let draw_list_stats = resolved_draw_plan
            .draw_list_stats
            .unwrap_or_else(|| draw_list.stats());
        let draw_plan = resolved_draw_plan.plan;
        let image_resource_stats =
            self.prepare_image_resources(draw_list, &draw_plan.image_upload_sources);
        let text_stats = self.text.prepare(
            &self.device,
            &self.queue,
            draw_list.surface_size,
            draw_list,
            &draw_plan.ops,
        );
        let render_stats = self.render_draw_list_to_surface(draw_list, &draw_plan, mode, damage)?;
        let mut batch_stats = draw_plan.stats;
        batch_stats.batch_plan_build_count = resolved_draw_plan.batch_plan_build_count;
        batch_stats.batch_plan_cache_hit_count = resolved_draw_plan.batch_plan_cache_hit_count;
        batch_stats.vertex_buffer_create_count =
            render_stats.draw_buffers.vertex_buffer_create_count;
        batch_stats.vertex_upload_bytes = render_stats.draw_buffers.vertex_upload_bytes;
        batch_stats.retained_cache_copy_bytes = render_stats.retained_cache_copy_bytes;
        Ok(WgpuUiSurfacePresentation {
            draw_list_stats,
            batch_stats,
            text_stats,
            image_resource_stats: Some(image_resource_stats),
            recorded_stats: Some(render_stats.recorded),
        })
    }

    fn resize_if_needed(&mut self, size: (u32, u32)) -> Result<(), RhiError> {
        if size != (self.config.width, self.config.height) {
            self.resize(size)?;
        }
        Ok(())
    }

    fn prepare_image_resources(
        &mut self,
        draw_list: &UiSurfaceDrawList,
        image_upload_sources: &[ImageUploadSource],
    ) -> WgpuUiImageResourceStats {
        let mut stats = WgpuUiImageResourceStats::default();
        let max_texture_dimension_2d = self.device.limits().max_texture_dimension_2d;
        let mut admission_saturated = false;
        for image_upload_source in image_upload_sources {
            let cache_key = image_upload_source.resource_key.as_str();
            if cache_key.is_empty() {
                continue;
            }
            if let Some(generation) = draw_list.generation() {
                if let Some(resource) = self.image_cache.get_mut(cache_key) {
                    if !image_upload_needs_write(
                        Some(generation),
                        resource.last_uploaded_generation,
                    ) {
                        resource.last_touched_present = self.present_index;
                        stats.prepare_cache_hit_count =
                            stats.prepare_cache_hit_count.saturating_add(1);
                        continue;
                    }
                }
            }
            for command_index in &image_upload_source.command_indices {
                stats.prepare_command_visit_count =
                    stats.prepare_command_visit_count.saturating_add(1);
                let Some(command) = draw_list.commands.get(*command_index) else {
                    continue;
                };
                let UiSurfaceCommandKind::Image { payload } = &command.kind else {
                    continue;
                };
                if payload.resource_key != cache_key {
                    continue;
                }
                let Some(rgba) = payload.rgba.as_deref() else {
                    if let Some(resource) = self.image_cache.get_mut(cache_key) {
                        resource.last_touched_present = self.present_index;
                    }
                    continue;
                };
                let Some(layout) =
                    image_payload_layout(payload.width, payload.height, max_texture_dimension_2d)
                else {
                    stats.invalid_payload_count = stats.invalid_payload_count.saturating_add(1);
                    self.invalidate_cached_image(cache_key);
                    continue;
                };
                if rgba.len() < layout.expected_len {
                    stats.invalid_payload_count = stats.invalid_payload_count.saturating_add(1);
                    self.invalidate_cached_image(cache_key);
                    continue;
                }
                let cached_size = self
                    .image_cache
                    .get(cache_key)
                    .map(|resource| resource.size);
                let replace = cached_size != Some((payload.width, payload.height));
                if replace {
                    if !self.admit_image_cache_resource(
                        cache_key,
                        layout.expected_len as u64,
                        image_upload_sources,
                        cached_size.is_none(),
                        &mut admission_saturated,
                        &mut stats,
                    ) {
                        self.invalidate_cached_image(cache_key);
                        continue;
                    }
                    let resource = WgpuUiImageResource::new(
                        &self.device,
                        &self.image_bind_group_layout,
                        &self.image_sampler,
                        (payload.width, payload.height),
                        layout.expected_len as u64,
                        self.present_index,
                    );
                    if let Some(cached) = self.image_cache.get_mut(cache_key) {
                        self.image_cache_bytes = self
                            .image_cache_bytes
                            .saturating_sub(cached.byte_size)
                            .saturating_add(resource.byte_size);
                        *cached = resource;
                    } else {
                        self.image_cache_bytes =
                            self.image_cache_bytes.saturating_add(resource.byte_size);
                        self.image_cache.insert(cache_key.to_owned(), resource);
                        stats.cache_key_allocation_count =
                            stats.cache_key_allocation_count.saturating_add(1);
                    }
                }
                if let Some(resource) = self.image_cache.get_mut(cache_key) {
                    resource.last_touched_present = self.present_index;
                    if image_upload_needs_write(
                        draw_list.generation(),
                        resource.last_uploaded_generation,
                    ) {
                        self.queue.write_texture(
                            resource.texture.as_image_copy(),
                            &rgba[..layout.expected_len],
                            wgpu::TexelCopyBufferLayout {
                                offset: 0,
                                bytes_per_row: Some(layout.bytes_per_row),
                                rows_per_image: Some(payload.height),
                            },
                            wgpu::Extent3d {
                                width: payload.width,
                                height: payload.height,
                                depth_or_array_layers: 1,
                            },
                        );
                        resource.last_uploaded_generation = draw_list.generation();
                        stats.upload_write_count = stats.upload_write_count.saturating_add(1);
                        stats.upload_bytes = stats
                            .upload_bytes
                            .saturating_add(layout.expected_len as u64);
                    }
                    break;
                }
            }
        }
        stats.cache_resident_bytes = self.image_cache_bytes;
        stats
    }

    fn admit_image_cache_resource(
        &mut self,
        cache_key: &str,
        required_bytes: u64,
        active_sources: &[ImageUploadSource],
        new_key: bool,
        admission_saturated: &mut bool,
        stats: &mut WgpuUiImageResourceStats,
    ) -> bool {
        if new_key && *admission_saturated {
            stats.cache_admission_reject_count =
                stats.cache_admission_reject_count.saturating_add(1);
            return false;
        }
        let replaced_bytes = self
            .image_cache
            .get(cache_key)
            .map_or(0, |resource| resource.byte_size);
        let entry_count_after = self.image_cache.len().saturating_add(usize::from(new_key));
        let cache_bytes_after = self
            .image_cache_bytes
            .saturating_sub(replaced_bytes)
            .saturating_add(required_bytes);
        let (action, visit_count) = image_cache_admission_plan(
            self.image_cache.iter().map(|(key, resource)| {
                let key_str = key.as_str();
                (
                    key_str,
                    resource.last_touched_present,
                    resource.byte_size,
                    active_sources
                        .binary_search_by(|source| source.resource_key.as_str().cmp(key_str))
                        .is_ok(),
                    key_str == cache_key,
                )
            }),
            entry_count_after,
            cache_bytes_after,
            MAX_UI_IMAGE_CACHE_ENTRIES,
            MAX_UI_IMAGE_CACHE_BYTES,
            required_bytes,
        );
        stats.cache_prune_visit_count = stats.cache_prune_visit_count.saturating_add(visit_count);
        match action {
            ImageCacheAdmissionAction::Admit { evict_keys } => {
                stats.cache_key_allocation_count = stats
                    .cache_key_allocation_count
                    .saturating_add(evict_keys.len() as u64);
                for key in evict_keys {
                    self.invalidate_cached_image(&key);
                }
                true
            }
            ImageCacheAdmissionAction::Reject { cache_saturated } => {
                *admission_saturated |= new_key && cache_saturated;
                stats.cache_admission_reject_count =
                    stats.cache_admission_reject_count.saturating_add(1);
                false
            }
        }
    }

    fn invalidate_cached_image(&mut self, cache_key: &str) -> bool {
        let Some(resource) = remove_cached_image(&mut self.image_cache, cache_key) else {
            return false;
        };
        self.image_cache_bytes = self.image_cache_bytes.saturating_sub(resource.byte_size);
        true
    }

    fn render_draw_list_to_surface(
        &mut self,
        draw_list: &UiSurfaceDrawList,
        draw_plan: &BatchDrawPlan,
        mode: SurfaceRenderMode,
        damage: Option<crate::rhi::UiSurfaceRect>,
    ) -> Result<WgpuUiSurfaceRenderStats, RhiError> {
        let surface_texture = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(surface_texture)
            | wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => surface_texture,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.config);
                return Ok(WgpuUiSurfaceRenderStats::default());
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(WgpuUiSurfaceRenderStats::default());
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
        let draw_ops = &draw_plan.ops;
        let resolved_buffers =
            self.compiled_draw_buffers
                .resolve(&self.device, &self.queue, draw_list, draw_plan);
        let buffers = resolved_buffers.buffers;
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("zircon-ui-surface-encoder"),
            });
        encoder.push_debug_group("zircon::UI");
        let mut render_stats = WgpuUiSurfaceRenderStats::default();

        match mode {
            SurfaceRenderMode::FullRedraw => {
                if let Some(retained_cache) = &mut self.retained_cache {
                    render_stats.add_recorded(record_draw_ops_to_view(
                        &mut encoder,
                        retained_cache.view(),
                        TargetLoad::ClearBlack,
                        draw_list.surface_size,
                        damage,
                        draw_ops,
                        &buffers,
                        &self.solid_pipeline,
                        &self.image_pipeline,
                        &self.image_cache,
                        &mut self.text,
                    ));
                    render_stats.retained_cache_copy_bytes = render_stats
                        .retained_cache_copy_bytes
                        .saturating_add(retained_cache.record_copy_to_surface(
                            &mut encoder,
                            &surface_texture.texture,
                            draw_list.surface_size,
                        ));
                    retained_cache.mark_initialized();
                } else {
                    render_stats.add_recorded(record_draw_ops_to_view(
                        &mut encoder,
                        &target_view,
                        TargetLoad::ClearBlack,
                        draw_list.surface_size,
                        damage,
                        draw_ops,
                        &buffers,
                        &self.solid_pipeline,
                        &self.image_pipeline,
                        &self.image_cache,
                        &mut self.text,
                    ));
                }
            }
            SurfaceRenderMode::DamagePatch => {
                let retained_cache = self.retained_cache.as_mut().ok_or_else(|| {
                    RhiError::SurfaceUnavailable(
                        "damage patch requested without a retained surface cache".to_string(),
                    )
                })?;
                render_stats.add_recorded(record_draw_ops_to_view(
                    &mut encoder,
                    retained_cache.view(),
                    TargetLoad::Load,
                    draw_list.surface_size,
                    damage,
                    draw_ops,
                    &buffers,
                    &self.solid_pipeline,
                    &self.image_pipeline,
                    &self.image_cache,
                    &mut self.text,
                ));
                render_stats.retained_cache_copy_bytes = render_stats
                    .retained_cache_copy_bytes
                    .saturating_add(retained_cache.record_copy_to_surface(
                        &mut encoder,
                        &surface_texture.texture,
                        draw_list.surface_size,
                    ));
                retained_cache.mark_initialized();
            }
        }

        encoder.pop_debug_group();
        self.queue.submit(Some(encoder.finish()));
        surface_texture.present();
        Ok(WgpuUiSurfaceRenderStats {
            draw_buffers: resolved_buffers.stats,
            ..render_stats
        })
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

fn retained_cache_copy_supported(surface_usage: wgpu::TextureUsages) -> bool {
    surface_usage.contains(wgpu::TextureUsages::COPY_DST)
}

fn render_damage(
    draw_list: &UiSurfaceDrawList,
    mode: SurfaceRenderMode,
) -> Option<crate::rhi::UiSurfaceRect> {
    (mode == SurfaceRenderMode::DamagePatch)
        .then_some(draw_list.damage)
        .flatten()
}

struct WgpuUiImageResource {
    texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
    size: (u32, u32),
    byte_size: u64,
    last_touched_present: u64,
    last_uploaded_generation: Option<u64>,
}

impl WgpuUiImageResource {
    fn new(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
        size: (u32, u32),
        byte_size: u64,
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
        Self {
            texture,
            bind_group,
            size,
            byte_size,
            last_touched_present,
            last_uploaded_generation: None,
        }
    }
}

fn image_upload_needs_write(
    draw_list_generation: Option<u64>,
    last_uploaded_generation: Option<u64>,
) -> bool {
    // An unversioned producer has not promised payload stability, so it must upload every time.
    draw_list_generation != last_uploaded_generation || draw_list_generation.is_none()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ImagePayloadLayout {
    expected_len: usize,
    bytes_per_row: u32,
}

fn image_payload_layout(
    width: u32,
    height: u32,
    max_texture_dimension_2d: u32,
) -> Option<ImagePayloadLayout> {
    if width == 0
        || height == 0
        || width > max_texture_dimension_2d
        || height > max_texture_dimension_2d
    {
        return None;
    }
    let bytes_per_row = width.checked_mul(4)?;
    let expected_len = u64::from(width)
        .checked_mul(u64::from(height))
        .and_then(|pixel_count| pixel_count.checked_mul(4))
        .and_then(|byte_count| usize::try_from(byte_count).ok())?;
    Some(ImagePayloadLayout {
        expected_len,
        bytes_per_row,
    })
}

fn remove_cached_image<T>(cache: &mut HashMap<String, T>, cache_key: &str) -> Option<T> {
    cache.remove(cache_key)
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ImageCacheAdmissionAction {
    Admit { evict_keys: Vec<String> },
    Reject { cache_saturated: bool },
}

fn image_cache_admission_plan<'a>(
    entries: impl Iterator<Item = (&'a str, u64, u64, bool, bool)>,
    cache_entry_count_after: usize,
    cache_bytes_after: u64,
    max_entries: usize,
    max_bytes: u64,
    required_bytes: u64,
) -> (ImageCacheAdmissionAction, u64) {
    if cache_entry_count_after <= max_entries && cache_bytes_after <= max_bytes {
        return (
            ImageCacheAdmissionAction::Admit {
                evict_keys: Vec::new(),
            },
            0,
        );
    }
    if max_entries == 0 || required_bytes > max_bytes {
        return (
            ImageCacheAdmissionAction::Reject {
                cache_saturated: false,
            },
            0,
        );
    }
    let mut visit_count = 0_u64;
    let mut candidates = Vec::new();
    for (key, last_touched_present, byte_size, active, target) in entries {
        visit_count = visit_count.saturating_add(1);
        if active || target {
            continue;
        }
        candidates.push((last_touched_present, key, byte_size));
    }
    candidates.sort_unstable_by_key(|(last_touched_present, key, _)| (*last_touched_present, *key));
    let mut retained_entries = cache_entry_count_after;
    let mut retained_bytes = cache_bytes_after;
    let mut evict_keys = Vec::new();
    for (_, key, byte_size) in candidates {
        if retained_entries <= max_entries && retained_bytes <= max_bytes {
            break;
        }
        retained_entries = retained_entries.saturating_sub(1);
        retained_bytes = retained_bytes.saturating_sub(byte_size);
        evict_keys.push(key.to_owned());
    }
    let action = if retained_entries <= max_entries && retained_bytes <= max_bytes {
        ImageCacheAdmissionAction::Admit { evict_keys }
    } else {
        ImageCacheAdmissionAction::Reject {
            cache_saturated: true,
        }
    };
    (action, visit_count)
}

#[cfg(test)]
mod tests;
