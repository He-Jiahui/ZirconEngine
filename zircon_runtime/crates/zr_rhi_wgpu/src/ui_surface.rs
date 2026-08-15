#[cfg(test)]
use crate::GPU_TIMESTAMP_REQUIRED_FEATURES;
use std::sync::Arc;

use crate::{GpuPassTimer, GpuReadbackQueue};
use zr_rhi::{
    RhiError, UiSurfaceDescriptor, UiSurfaceDrawList, UiSurfaceImageResourceTable,
    UiSurfacePresentOutcome, UiSurfacePresentStats, UiSurfacePresenter,
};

mod batching;
mod geometry;
mod image_cache;
mod pipeline;
mod presentation;
mod render_pass;
mod retained_cache;
mod shared_image_registry;
mod surface_setup;
mod text;

use batching::{BatchDrawPlanStats, CompiledUiBatchPlanCache};
use image_cache::{WgpuUiImageCache, WgpuUiImageResourceStats};
use pipeline::{
    create_image_bind_group_layout, create_image_pipeline, create_image_sampler,
    create_solid_instance_pipeline, create_solid_pipeline,
};
use render_pass::{WgpuUiDrawBufferCache, WgpuUiDrawBufferStats, WgpuUiRecordedDrawStats};
use retained_cache::WgpuRetainedSurfaceCache;
pub use shared_image_registry::WgpuUiSharedImageRegistry;
use surface_setup::{configure_surface, create_surface, instance_descriptor, request_device};
use text::{WgpuUiTextPrepareStats, WgpuUiTextRenderer};

#[cfg(test)]
use presentation::{
    render_damage, retryable_surface_outcome, retryable_surface_presentation, surface_render_mode,
    SurfaceRenderMode,
};

const UI_GPU_TIMER_MAX_PASSES: u32 = 1;
const UI_GPU_TIMER_PASS_NAME: &str = "ui.surface";

#[cfg(test)]
use surface_setup::{
    choose_alpha_mode, choose_surface_format, choose_surface_usage, requested_device_features,
};

#[cfg(test)]
use image_cache::{
    image_cache_admission_plan, image_payload_layout, image_upload_needs_write,
    remove_cached_image, take_image_source_pixels, ImageCacheAdmissionAction, ImagePayloadLayout,
};

/// Owned WGPU state that lets a native UI surface share the runtime renderer's device.
///
/// The context is deliberately made of cloned WGPU handles rather than raw native pointers.
/// Its creator must keep the instance, adapter, device, and queue from one negotiated backend;
/// that invariant makes imported render products and UI sampling use the same WGPU device.
#[derive(Clone)]
pub struct WgpuUiSurfaceContext {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    shared_image_registry: Arc<WgpuUiSharedImageRegistry>,
}

impl WgpuUiSurfaceContext {
    pub fn new(
        instance: wgpu::Instance,
        adapter: wgpu::Adapter,
        device: wgpu::Device,
        queue: wgpu::Queue,
    ) -> Self {
        Self::new_with_shared_image_registry(
            instance,
            adapter,
            device,
            queue,
            Arc::new(WgpuUiSharedImageRegistry::default()),
        )
    }

    pub fn new_with_shared_image_registry(
        instance: wgpu::Instance,
        adapter: wgpu::Adapter,
        device: wgpu::Device,
        queue: wgpu::Queue,
        shared_image_registry: Arc<WgpuUiSharedImageRegistry>,
    ) -> Self {
        Self {
            instance,
            adapter,
            device,
            queue,
            shared_image_registry,
        }
    }

    /// Copies a renderer-owned output into a texture whose identity is stable for one UI product.
    ///
    /// Both submissions use this context's queue, so queue order is the GPU lifetime fence: a UI
    /// present can only sample the copied product after the producer and this copy have executed.
    /// No staging buffer, mapping operation, or CPU pixel clone is involved.
    pub fn copy_texture_for_external_image(
        &self,
        source: &wgpu::Texture,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        generation: u64,
    ) -> WgpuUiExternalImage {
        let byte_space_view_format = byte_space_sample_view_format(format);
        let view_formats = byte_space_view_format
            .as_ref()
            .map(std::slice::from_ref)
            .unwrap_or_default();
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-ui-external-product"),
            size: wgpu::Extent3d {
                width: width.max(1),
                height: height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats,
        });
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("zircon-ui-external-product-copy"),
            });
        encoder.copy_texture_to_texture(
            source.as_image_copy(),
            texture.as_image_copy(),
            wgpu::Extent3d {
                width: width.max(1),
                height: height.max(1),
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit(Some(encoder.finish()));
        WgpuUiExternalImage::new_with_sample_view_format(
            texture,
            width,
            height,
            generation,
            WgpuUiExternalImageAlphaMode::Opaque,
            byte_space_view_format,
        )
    }
}

/// The retained UI composites byte-space image payloads. Sampling an sRGB product through its
/// default view would decode it before the UI pass and make the direct path darker than readback.
fn byte_space_sample_view_format(format: wgpu::TextureFormat) -> Option<wgpu::TextureFormat> {
    match format {
        wgpu::TextureFormat::Rgba8UnormSrgb => Some(wgpu::TextureFormat::Rgba8Unorm),
        wgpu::TextureFormat::Bgra8UnormSrgb => Some(wgpu::TextureFormat::Bgra8Unorm),
        _ => None,
    }
}

/// Alpha representation accepted by the premultiplied UI image pipeline.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WgpuUiExternalImageAlphaMode {
    /// Every sampled texel has alpha one, so straight and premultiplied RGB are equivalent.
    Opaque,
    /// RGB channels have already been multiplied by alpha before any linear filtering.
    Premultiplied,
}

/// A generation-stable texture product that can be sampled by a UI surface sharing its device.
///
/// The owner must retain this value until every UI present that resolves `generation` has been
/// submitted. Because the renderer and UI surface use one queue, their submissions establish the
/// required visibility order without a CPU copy or a raw native-handle lifetime escape.
#[derive(Clone)]
pub struct WgpuUiExternalImage {
    texture: wgpu::Texture,
    width: u32,
    height: u32,
    generation: u64,
    alpha_mode: WgpuUiExternalImageAlphaMode,
    sample_view_format: Option<wgpu::TextureFormat>,
}

impl WgpuUiExternalImage {
    /// Declares a texture whose sampled alpha is always one.
    pub fn new_opaque(texture: wgpu::Texture, width: u32, height: u32, generation: u64) -> Self {
        Self::new_with_sample_view_format(
            texture,
            width,
            height,
            generation,
            WgpuUiExternalImageAlphaMode::Opaque,
            None,
        )
    }

    /// Declares a texture whose RGB channels are already multiplied by alpha.
    pub fn new_premultiplied(
        texture: wgpu::Texture,
        width: u32,
        height: u32,
        generation: u64,
    ) -> Self {
        Self::new_with_sample_view_format(
            texture,
            width,
            height,
            generation,
            WgpuUiExternalImageAlphaMode::Premultiplied,
            None,
        )
    }

    fn new_with_sample_view_format(
        texture: wgpu::Texture,
        width: u32,
        height: u32,
        generation: u64,
        alpha_mode: WgpuUiExternalImageAlphaMode,
        sample_view_format: Option<wgpu::TextureFormat>,
    ) -> Self {
        Self {
            texture,
            width,
            height,
            generation,
            alpha_mode,
            sample_view_format,
        }
    }

    /// Returns the alpha representation declared by the producer.
    pub const fn alpha_mode(&self) -> WgpuUiExternalImageAlphaMode {
        self.alpha_mode
    }

    fn matches_generation(&self, generation: u64) -> bool {
        self.generation == generation && self.width != 0 && self.height != 0
    }

    fn create_sample_view(&self) -> wgpu::TextureView {
        self.texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("zircon-ui-external-image-view"),
            format: self.sample_view_format,
            ..Default::default()
        })
    }
}

/// Resolves retained UI image identities to products from the runtime render backend.
///
/// Implementations must return textures created by the exact [`WgpuUiSurfaceContext`] device
/// supplied to the presenter. Returning `None` selects the ordinary CPU image-cache fallback.
pub trait WgpuUiSurfaceExternalImageProvider: Send + Sync {
    fn resolve(&self, resource_key: &str, generation: u64) -> Option<WgpuUiExternalImage>;

    /// Records that the UI cache accepted a resolved texture for sampling.
    fn confirm_resident(&self, _resource_key: &str, _generation: u64) {}
}

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
            WgpuUiSurfaceBackend::Native(Box::new(WgpuUiSurfaceRenderer::new_owned(descriptor)?))
        } else {
            WgpuUiSurfaceBackend::Headless(CompiledUiBatchPlanCache::default())
        };
        let mut last_stats = UiSurfacePresentStats::default();
        last_stats.surface_size = descriptor.clamped_size();
        Ok(Self {
            descriptor,
            backend,
            last_stats,
            presented_frame_count: 0,
        })
    }

    /// Creates a presenter backed by an already-negotiated runtime WGPU device.
    ///
    /// Headless descriptors keep their deterministic no-device behavior. Native descriptors use
    /// the supplied context to create and configure their surface, so render product textures can
    /// later be sampled without a CPU readback or cross-device copy.
    pub fn new_with_context(
        descriptor: UiSurfaceDescriptor,
        context: WgpuUiSurfaceContext,
    ) -> Result<Self, RhiError> {
        Self::new_with_context_and_external_images(descriptor, context, None)
    }

    /// Creates a shared-device presenter that can sample runtime-owned texture products.
    pub fn new_with_context_and_external_images(
        descriptor: UiSurfaceDescriptor,
        context: WgpuUiSurfaceContext,
        external_images: Option<Arc<dyn WgpuUiSurfaceExternalImageProvider>>,
    ) -> Result<Self, RhiError> {
        descriptor.validate()?;
        let backend = if descriptor.target.is_some() {
            WgpuUiSurfaceBackend::Native(Box::new(WgpuUiSurfaceRenderer::new_with_context(
                descriptor,
                context,
                external_images,
            )?))
        } else {
            WgpuUiSurfaceBackend::Headless(CompiledUiBatchPlanCache::default())
        };
        let mut last_stats = UiSurfacePresentStats::default();
        last_stats.surface_size = descriptor.clamped_size();
        Ok(Self {
            descriptor,
            backend,
            last_stats,
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
        let size = (width.max(1), height.max(1));
        if size == self.descriptor.clamped_size() {
            return Ok(());
        }
        self.descriptor.width = size.0;
        self.descriptor.height = size.1;
        if let WgpuUiSurfaceBackend::Native(renderer) = &mut self.backend {
            renderer.resize(size)?;
        }
        self.last_stats.surface_size = size;
        Ok(())
    }

    fn is_image_resource_resident(&self, resource_key: &str, generation: u64) -> bool {
        match &self.backend {
            WgpuUiSurfaceBackend::Native(renderer) => {
                renderer.image_cache.is_resident(resource_key, generation)
                    || renderer.external_images.as_ref().is_some_and(|provider| {
                        provider
                            .resolve(resource_key, generation)
                            .is_some_and(|image| image.matches_generation(generation))
                    })
            }
            WgpuUiSurfaceBackend::Headless(_) => false,
        }
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
                    outcome: UiSurfacePresentOutcome::Submitted,
                    draw_list_stats: resolved_draw_plan
                        .draw_list_stats
                        .unwrap_or_else(|| draw_list.stats()),
                    batch_stats,
                    text_stats: WgpuUiTextPrepareStats::default(),
                    image_resource_stats: None,
                    recorded_stats: None,
                    gpu_timestamp_supported: false,
                    gpu_time_us: None,
                    gpu_profile_latency_frames: 0,
                }
            }
        };

        let mut stats = presentation.draw_list_stats;
        stats.outcome = presentation.outcome;
        stats.compiled_draw_calls = presentation.batch_stats.draw_calls;
        stats.compiled_visible_draw_item_count = presentation.batch_stats.visible_draw_item_count;
        stats.compiled_solid_vertex_count = presentation.batch_stats.solid_vertex_count;
        stats.compiled_solid_instance_count = presentation.batch_stats.solid_instance_count;
        stats.compiled_image_vertex_count = presentation.batch_stats.image_vertex_count;
        stats.compiled_batch_layer_count = presentation.batch_stats.batch_layer_count;
        stats.compiled_batch_dependency_count = presentation.batch_stats.batch_dependency_count;
        stats.compiled_batch_merge_count = presentation.batch_stats.batch_merge_count;
        stats.draw_calls = presentation.batch_stats.draw_calls;
        stats.render_pass_count = presentation.batch_stats.render_pass_count;
        stats.visible_draw_item_count = presentation.batch_stats.visible_draw_item_count;
        stats.batch_merge_count = presentation.batch_stats.batch_merge_count;
        stats.solid_vertex_count = presentation.batch_stats.solid_vertex_count;
        stats.solid_instance_count = presentation.batch_stats.solid_instance_count;
        stats.image_vertex_count = presentation.batch_stats.image_vertex_count;
        stats.batch_layer_count = presentation.batch_stats.batch_layer_count;
        stats.batch_dependency_count = presentation.batch_stats.batch_dependency_count;
        stats.overlap_candidate_count = presentation.batch_stats.overlap_candidate_count;
        stats.batch_plan_build_count = presentation.batch_stats.batch_plan_build_count;
        stats.batch_plan_cache_hit_count = presentation.batch_stats.batch_plan_cache_hit_count;
        stats.vertex_buffer_create_count = presentation.batch_stats.vertex_buffer_create_count;
        stats.vertex_upload_bytes = presentation.batch_stats.vertex_upload_bytes;
        stats.retained_cache_copy_bytes = presentation.batch_stats.retained_cache_copy_bytes;
        stats.gpu_timestamp_supported = presentation.gpu_timestamp_supported;
        stats.gpu_time_us = presentation.gpu_time_us;
        stats.gpu_profile_latency_frames = presentation.gpu_profile_latency_frames;
        if let Some(recorded) = presentation.recorded_stats {
            stats.draw_calls = recorded.draw_calls;
            stats.render_pass_count = recorded.render_pass_count;
            stats.visible_draw_item_count = recorded.visible_draw_item_count;
            stats.solid_vertex_count = recorded.solid_vertex_count;
            stats.solid_instance_count = recorded.solid_instance_count;
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
            stats.image_shared_resolve_count = image_resource_stats.shared_resolve_count;
            stats.image_shared_upload_write_count = image_resource_stats.shared_upload_write_count;
            stats.image_shared_upload_bytes = image_resource_stats.shared_upload_bytes;
            stats.image_shared_resident_bytes = image_resource_stats.shared_resident_bytes;
            stats.image_cache_key_allocation_count =
                image_resource_stats.cache_key_allocation_count;
            stats.image_cache_prune_visit_count = image_resource_stats.cache_prune_visit_count;
            stats.image_cache_admission_reject_count =
                image_resource_stats.cache_admission_reject_count;
            stats.image_invalid_payload_count = image_resource_stats.invalid_payload_count;
            stats.image_cache_resident_bytes = image_resource_stats.cache_resident_bytes;
            stats.image_cache_cpu_resident_bytes = image_resource_stats.cpu_resident_bytes;
            stats.image_prepare_command_visit_count =
                image_resource_stats.prepare_command_visit_count;
            stats.image_prepare_cache_hit_count = image_resource_stats.prepare_cache_hit_count;
        }
        self.presented_frame_count =
            advance_presented_frame_count(self.presented_frame_count, presentation.outcome);
        stats.presented_frame_count = self.presented_frame_count;
        if presentation.outcome.is_submitted() {
            self.last_stats = stats;
        }
        Ok(stats)
    }

    fn present_owned(
        &mut self,
        mut draw_list: UiSurfaceDrawList,
    ) -> Result<UiSurfacePresentStats, RhiError> {
        if let WgpuUiSurfaceBackend::Native(renderer) = &mut self.backend {
            renderer.stage_image_resources(draw_list.take_image_resources());
        }
        self.present(&draw_list)
    }

    fn last_present_stats(&self) -> UiSurfacePresentStats {
        self.last_stats
    }
}

struct WgpuUiSurfacePresentation {
    outcome: UiSurfacePresentOutcome,
    draw_list_stats: UiSurfacePresentStats,
    batch_stats: BatchDrawPlanStats,
    text_stats: WgpuUiTextPrepareStats,
    image_resource_stats: Option<WgpuUiImageResourceStats>,
    recorded_stats: Option<WgpuUiRecordedDrawStats>,
    gpu_timestamp_supported: bool,
    gpu_time_us: Option<u64>,
    gpu_profile_latency_frames: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct WgpuUiSurfaceRenderStats {
    outcome: UiSurfacePresentOutcome,
    draw_buffers: WgpuUiDrawBufferStats,
    recorded: WgpuUiRecordedDrawStats,
    retained_cache_copy_bytes: u64,
    gpu_timestamp_supported: bool,
    gpu_time_us: Option<u64>,
    gpu_profile_latency_frames: u32,
}

fn advance_presented_frame_count(current: u64, outcome: UiSurfacePresentOutcome) -> u64 {
    if outcome.is_submitted() {
        current.saturating_add(1)
    } else {
        current
    }
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
        self.recorded.solid_instance_count = self
            .recorded
            .solid_instance_count
            .saturating_add(recorded.solid_instance_count);
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
    solid_instance_pipeline: wgpu::RenderPipeline,
    image_pipeline: wgpu::RenderPipeline,
    image_bind_group_layout: wgpu::BindGroupLayout,
    image_sampler: wgpu::Sampler,
    retained_cache: Option<WgpuRetainedSurfaceCache>,
    image_cache: WgpuUiImageCache,
    shared_image_registry: Arc<WgpuUiSharedImageRegistry>,
    external_images: Option<Arc<dyn WgpuUiSurfaceExternalImageProvider>>,
    pending_image_resources: UiSurfaceImageResourceTable,
    text: WgpuUiTextRenderer,
    gpu_readback_queue: GpuReadbackQueue,
    gpu_pass_timer: Option<GpuPassTimer>,
    compiled_batch_plan: CompiledUiBatchPlanCache,
    compiled_draw_buffers: WgpuUiDrawBufferCache,
    present_index: u64,
}

impl WgpuUiSurfaceRenderer {
    fn stage_image_resources(&mut self, image_resources: UiSurfaceImageResourceTable) {
        self.pending_image_resources = image_resources;
    }

    fn new_owned(descriptor: UiSurfaceDescriptor) -> Result<Self, RhiError> {
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
        let (device, queue) = request_device(&adapter, descriptor.allow_gpu_timing)?;
        Self::from_surface(
            descriptor,
            WgpuUiSurfaceContext::new(instance, adapter, device, queue),
            surface,
            None,
        )
    }

    fn new_with_context(
        descriptor: UiSurfaceDescriptor,
        context: WgpuUiSurfaceContext,
        external_images: Option<Arc<dyn WgpuUiSurfaceExternalImageProvider>>,
    ) -> Result<Self, RhiError> {
        let target = descriptor
            .target
            .ok_or_else(|| RhiError::SurfaceUnavailable("missing native surface target".into()))?;
        let surface = create_surface(&context.instance, target)?;
        Self::from_surface(descriptor, context, surface, external_images)
    }

    fn from_surface(
        descriptor: UiSurfaceDescriptor,
        context: WgpuUiSurfaceContext,
        surface: wgpu::Surface<'static>,
        external_images: Option<Arc<dyn WgpuUiSurfaceExternalImageProvider>>,
    ) -> Result<Self, RhiError> {
        let size = descriptor.clamped_size();
        let config = configure_surface(&surface, &context.adapter, &context.device, size)?;
        let solid_pipeline = create_solid_pipeline(&context.device, config.format);
        let solid_instance_pipeline =
            create_solid_instance_pipeline(&context.device, config.format);
        let image_bind_group_layout = create_image_bind_group_layout(&context.device);
        let image_sampler = create_image_sampler(&context.device);
        let image_pipeline =
            create_image_pipeline(&context.device, config.format, &image_bind_group_layout);
        let retained_cache = retained_cache_copy_supported(config.usage)
            .then(|| WgpuRetainedSurfaceCache::new(&context.device, config.format, size));
        let text = WgpuUiTextRenderer::new(&context.device, &context.queue, config.format);
        let gpu_pass_timer = descriptor
            .allow_gpu_timing
            .then(|| {
                GpuPassTimer::try_new(&context.device, &context.queue, UI_GPU_TIMER_MAX_PASSES)
            })
            .flatten();
        let gpu_readback_queue = GpuReadbackQueue::new(&context.device);

        Ok(Self {
            _instance: context.instance,
            _adapter: context.adapter,
            device: context.device,
            queue: context.queue,
            surface,
            config,
            solid_pipeline,
            solid_instance_pipeline,
            image_pipeline,
            image_bind_group_layout,
            image_sampler,
            retained_cache,
            image_cache: WgpuUiImageCache::default(),
            shared_image_registry: context.shared_image_registry,
            external_images,
            pending_image_resources: UiSurfaceImageResourceTable::default(),
            text,
            gpu_readback_queue,
            gpu_pass_timer,
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
        Ok(())
    }
}

fn retained_cache_copy_supported(surface_usage: wgpu::TextureUsages) -> bool {
    surface_usage.contains(wgpu::TextureUsages::COPY_DST)
}

#[cfg(test)]
mod tests;
