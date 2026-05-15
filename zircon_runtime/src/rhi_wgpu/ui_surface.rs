use std::collections::HashMap;
use std::num::NonZeroIsize;

use wgpu::util::DeviceExt;

use crate::core::framework::render::RenderNativeSurfaceTarget;
use crate::rhi::{
    RhiError, UiSurfaceCommandKind, UiSurfaceDescriptor, UiSurfaceDrawList, UiSurfacePresentStats,
    UiSurfacePresenter,
};

mod batching;
mod geometry;
mod pipeline;
mod text;

use batching::{batch_draw_plan, BatchDrawPlanStats, DrawOp};
use geometry::command_effective_rect;
use pipeline::{
    create_image_bind_group_layout, create_image_pipeline, create_image_sampler,
    create_solid_pipeline, WgpuBlitResources,
};
use text::WgpuUiTextRenderer;

const SURFACE_FRAME_LATENCY: u32 = 2;
const OFFSCREEN_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
const MAX_UI_IMAGE_CACHE_ENTRIES: usize = 256;

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
        let batch_stats = match &mut self.backend {
            WgpuUiSurfaceBackend::Native(renderer) => renderer.present(draw_list)?,
            WgpuUiSurfaceBackend::Headless => batch_draw_plan(draw_list).stats,
        };

        let mut stats = draw_list.stats();
        stats.draw_calls = batch_stats.draw_calls;
        stats.visible_draw_item_count = batch_stats.visible_draw_item_count;
        stats.batch_layer_count = batch_stats.batch_layer_count;
        stats.batch_dependency_count = batch_stats.batch_dependency_count;
        self.presented_frame_count = self.presented_frame_count.saturating_add(1);
        stats.presented_frame_count = self.presented_frame_count;
        self.last_stats = stats.clone();
        Ok(stats)
    }

    fn last_present_stats(&self) -> UiSurfacePresentStats {
        self.last_stats.clone()
    }
}

struct WgpuUiSurfaceRenderer {
    _instance: wgpu::Instance,
    _adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    offscreen: WgpuOffscreenTarget,
    solid_pipeline: wgpu::RenderPipeline,
    image_pipeline: wgpu::RenderPipeline,
    image_bind_group_layout: wgpu::BindGroupLayout,
    image_sampler: wgpu::Sampler,
    image_cache: HashMap<String, WgpuUiImageResource>,
    text: WgpuUiTextRenderer,
    blit: WgpuBlitResources,
    offscreen_initialized: bool,
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
        let offscreen = WgpuOffscreenTarget::new(&device, size);
        let solid_pipeline = create_solid_pipeline(&device, OFFSCREEN_FORMAT);
        let image_bind_group_layout = create_image_bind_group_layout(&device);
        let image_sampler = create_image_sampler(&device);
        let image_pipeline =
            create_image_pipeline(&device, OFFSCREEN_FORMAT, &image_bind_group_layout);
        let text = WgpuUiTextRenderer::new(&device, &queue, OFFSCREEN_FORMAT);
        let blit = WgpuBlitResources::new(&device, config.format, &image_bind_group_layout);

        Ok(Self {
            _instance: instance,
            _adapter: adapter,
            device,
            queue,
            surface,
            config,
            offscreen,
            solid_pipeline,
            image_pipeline,
            image_bind_group_layout,
            image_sampler,
            image_cache: HashMap::new(),
            text,
            blit,
            offscreen_initialized: false,
            present_index: 0,
        })
    }

    fn resize(&mut self, size: (u32, u32)) -> Result<(), RhiError> {
        let size = (size.0.max(1), size.1.max(1));
        self.config.width = size.0;
        self.config.height = size.1;
        self.surface.configure(&self.device, &self.config);
        self.offscreen = WgpuOffscreenTarget::new(&self.device, size);
        self.offscreen_initialized = false;
        Ok(())
    }

    fn present(&mut self, draw_list: &UiSurfaceDrawList) -> Result<BatchDrawPlanStats, RhiError> {
        self.resize_if_needed(draw_list.surface_size)?;
        self.present_index = self.present_index.saturating_add(1);
        let draw_plan = batch_draw_plan(draw_list);
        self.prepare_image_resources(draw_list);
        self.text.prepare(
            &self.device,
            &self.queue,
            draw_list.surface_size,
            draw_list,
            &draw_plan.ops,
        );
        self.render_draw_list_to_offscreen(draw_list, &draw_plan.ops);
        self.prune_image_cache();
        self.blit_offscreen_to_surface()?;
        Ok(draw_plan.stats)
    }

    fn resize_if_needed(&mut self, size: (u32, u32)) -> Result<(), RhiError> {
        if size != (self.config.width, self.config.height) {
            self.resize(size)?;
        }
        Ok(())
    }

    fn prepare_image_resources(&mut self, draw_list: &UiSurfaceDrawList) {
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

    fn render_draw_list_to_offscreen(
        &mut self,
        draw_list: &UiSurfaceDrawList,
        draw_ops: &[DrawOp],
    ) {
        let solid_vertices = draw_ops
            .iter()
            .filter_map(|op| match op {
                DrawOp::Solid(draw) => Some(draw.vertices.as_slice()),
                DrawOp::Image(_) => None,
                DrawOp::Text(_) => None,
            })
            .flatten()
            .copied()
            .collect::<Vec<_>>();
        let image_vertices = draw_ops
            .iter()
            .filter_map(|op| match op {
                DrawOp::Solid(_) => None,
                DrawOp::Image(draw) => Some(draw.vertices.as_slice()),
                DrawOp::Text(_) => None,
            })
            .flatten()
            .copied()
            .collect::<Vec<_>>();
        let solid_buffer = (!solid_vertices.is_empty()).then(|| {
            self.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("zircon-ui-solid-vertices"),
                    contents: bytemuck::cast_slice(&solid_vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                })
        });
        let image_buffer = (!image_vertices.is_empty()).then(|| {
            self.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("zircon-ui-image-vertices"),
                    contents: bytemuck::cast_slice(&image_vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                })
        });
        let retain_existing = draw_list.damage.is_some() && self.offscreen_initialized;
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("zircon-ui-offscreen-encoder"),
            });
        if draw_ops.is_empty() {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("zircon-ui-offscreen-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.offscreen.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: if retain_existing {
                            wgpu::LoadOp::Load
                        } else {
                            wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT)
                        },
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_viewport(
                0.0,
                0.0,
                draw_list.surface_size.0 as f32,
                draw_list.surface_size.1 as f32,
                0.0,
                1.0,
            );
        } else {
            let mut first_pass = true;
            let mut op_index = 0;
            while op_index < draw_ops.len() {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("zircon-ui-offscreen-pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &self.offscreen.view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: if first_pass && !retain_existing {
                                wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT)
                            } else {
                                wgpu::LoadOp::Load
                            },
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
                first_pass = false;
                match &draw_ops[op_index] {
                    DrawOp::Solid(_) => {
                        let Some(buffer) = solid_buffer.as_ref() else {
                            op_index += 1;
                            continue;
                        };
                        pass.set_pipeline(&self.solid_pipeline);
                        pass.set_vertex_buffer(0, buffer.slice(..));
                        let DrawOp::Solid(draw) = &draw_ops[op_index] else {
                            unreachable!("draw op kind checked above");
                        };
                        pass.draw(draw.vertex_start..draw.vertex_end, 0..1);
                        op_index += 1;
                    }
                    DrawOp::Image(_) => {
                        let Some(buffer) = image_buffer.as_ref() else {
                            op_index += 1;
                            continue;
                        };
                        pass.set_pipeline(&self.image_pipeline);
                        pass.set_vertex_buffer(0, buffer.slice(..));
                        let DrawOp::Image(draw) = &draw_ops[op_index] else {
                            unreachable!("draw op kind checked above");
                        };
                        if let Some(resource) = self.image_cache.get(&draw.resource_key) {
                            pass.set_bind_group(0, &resource.bind_group, &[]);
                            pass.draw(draw.vertex_start..draw.vertex_end, 0..1);
                        }
                        op_index += 1;
                    }
                    DrawOp::Text(draw) => {
                        self.text.render_batch(draw.batch_index, &mut pass);
                        op_index += 1;
                    }
                }
            }
        }
        self.queue.submit(Some(encoder.finish()));
        self.offscreen_initialized = true;
    }

    fn blit_offscreen_to_surface(&mut self) -> Result<(), RhiError> {
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
        self.blit.blit(
            &self.device,
            &self.queue,
            &self.offscreen.view,
            &target_view,
            (self.config.width, self.config.height),
        );
        surface_texture.present();
        Ok(())
    }
}

struct WgpuOffscreenTarget {
    view: wgpu::TextureView,
}

impl WgpuOffscreenTarget {
    fn new(device: &wgpu::Device, size: (u32, u32)) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-ui-offscreen"),
            size: wgpu::Extent3d {
                width: size.0.max(1),
                height: size.1.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: OFFSCREEN_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self { view }
    }
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
            format: OFFSCREEN_FORMAT,
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

fn configure_surface(
    surface: &wgpu::Surface<'static>,
    adapter: &wgpu::Adapter,
    device: &wgpu::Device,
    size: (u32, u32),
) -> Result<wgpu::SurfaceConfiguration, RhiError> {
    let caps = surface.get_capabilities(adapter);
    let Some(format) = choose_surface_format(&caps.formats) else {
        return Err(RhiError::SurfaceUnavailable(
            "surface has no compatible formats".to_string(),
        ));
    };
    let Some(present_mode) = choose_present_mode(&caps.present_modes) else {
        return Err(RhiError::SurfaceUnavailable(
            "surface has no compatible present modes".to_string(),
        ));
    };
    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format,
        width: size.0.max(1),
        height: size.1.max(1),
        present_mode,
        desired_maximum_frame_latency: SURFACE_FRAME_LATENCY,
        alpha_mode: caps
            .alpha_modes
            .first()
            .copied()
            .unwrap_or(wgpu::CompositeAlphaMode::Auto),
        view_formats: vec![],
    };
    surface.configure(device, &config);
    Ok(config)
}

fn choose_surface_format(formats: &[wgpu::TextureFormat]) -> Option<wgpu::TextureFormat> {
    [
        wgpu::TextureFormat::Bgra8UnormSrgb,
        wgpu::TextureFormat::Rgba8UnormSrgb,
        wgpu::TextureFormat::Bgra8Unorm,
        wgpu::TextureFormat::Rgba8Unorm,
    ]
    .into_iter()
    .find(|format| formats.contains(format))
    .or_else(|| formats.first().copied())
}

fn choose_present_mode(present_modes: &[wgpu::PresentMode]) -> Option<wgpu::PresentMode> {
    if present_modes.contains(&wgpu::PresentMode::AutoVsync) {
        Some(wgpu::PresentMode::AutoVsync)
    } else if present_modes.contains(&wgpu::PresentMode::Fifo) {
        Some(wgpu::PresentMode::Fifo)
    } else {
        present_modes.first().copied()
    }
}

fn request_device(adapter: &wgpu::Adapter) -> Result<(wgpu::Device, wgpu::Queue), RhiError> {
    let mut requested_features = wgpu::Features::empty();
    let adapter_features = adapter.features();
    if adapter_features.contains(wgpu::Features::INDIRECT_FIRST_INSTANCE) {
        requested_features |= wgpu::Features::INDIRECT_FIRST_INSTANCE;
    }
    pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("zircon-ui-device"),
        required_features: requested_features,
        required_limits: wgpu::Limits::default(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
    }))
    .map_err(|error| RhiError::SurfaceUnavailable(error.to_string()))
}

fn instance_descriptor() -> wgpu::InstanceDescriptor {
    let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
    descriptor.backends = std::env::var("WGPU_BACKEND")
        .ok()
        .as_deref()
        .map(wgpu::Backends::from_comma_list)
        .unwrap_or_default();
    descriptor.flags = wgpu::InstanceFlags::from_build_config();
    if let Ok(debug) = std::env::var("WGPU_DEBUG") {
        descriptor
            .flags
            .set(wgpu::InstanceFlags::DEBUG, debug != "0");
    }
    if let Ok(validation) = std::env::var("WGPU_VALIDATION") {
        descriptor
            .flags
            .set(wgpu::InstanceFlags::VALIDATION, validation != "0");
    }
    descriptor.backend_options = wgpu::BackendOptions::from_env_or_default();
    descriptor
}

#[cfg(target_os = "windows")]
fn create_surface(
    instance: &wgpu::Instance,
    target: RenderNativeSurfaceTarget,
) -> Result<wgpu::Surface<'static>, RhiError> {
    match target {
        RenderNativeSurfaceTarget::Win32 { hwnd, hinstance } => {
            let hwnd = required_nonzero_isize(hwnd, "invalid win32 hwnd")?;
            let mut window = wgpu::rwh::Win32WindowHandle::new(hwnd);
            window.hinstance = optional_nonzero_isize(hinstance)?;
            let raw_window_handle = wgpu::rwh::RawWindowHandle::Win32(window);
            let raw_display_handle =
                wgpu::rwh::RawDisplayHandle::Windows(wgpu::rwh::WindowsDisplayHandle::new());
            let target = wgpu::SurfaceTargetUnsafe::RawHandle {
                raw_display_handle: Some(raw_display_handle),
                raw_window_handle,
            };
            // The editor host owns the native window lifetime; runtime receives raw handles and
            // therefore must create the surface through wgpu's raw-handle entrypoint.
            unsafe { instance.create_surface_unsafe(target) }
                .map_err(|error| RhiError::SurfaceUnavailable(error.to_string()))
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn create_surface(
    _instance: &wgpu::Instance,
    target: RenderNativeSurfaceTarget,
) -> Result<wgpu::Surface<'static>, RhiError> {
    match target {
        RenderNativeSurfaceTarget::Win32 { .. } => Err(RhiError::SurfaceUnavailable(
            "win32 retained UI surfaces are only supported on windows".to_string(),
        )),
    }
}

fn required_nonzero_isize(value: u64, error: &'static str) -> Result<NonZeroIsize, RhiError> {
    if value == 0 || value > isize::MAX as u64 {
        return Err(RhiError::SurfaceUnavailable(error.to_string()));
    }
    Ok(NonZeroIsize::new(value as isize).expect("value checked above"))
}

fn optional_nonzero_isize(value: Option<u64>) -> Result<Option<NonZeroIsize>, RhiError> {
    value
        .map(|value| required_nonzero_isize(value, "invalid win32 hinstance"))
        .transpose()
}

#[cfg(test)]
mod tests {
    use crate::rhi::{
        UiSurfaceCommand, UiSurfaceCommandKind, UiSurfaceDrawList, UiSurfaceImagePayload,
        UiSurfaceRect,
    };

    use super::*;

    #[test]
    fn wgpu_ui_surface_presenter_records_present_stats() {
        let mut presenter = WgpuUiSurfacePresenter::new_headless(32, 16);
        let draw_list = UiSurfaceDrawList::new(
            (32, 16),
            None,
            vec![UiSurfaceCommand {
                z_index: 0,
                frame: UiSurfaceRect::new(0.0, 0.0, 16.0, 8.0),
                clip: None,
                kind: UiSurfaceCommandKind::Quad {
                    color: [1, 2, 3, 255],
                    corner_radius: 0.0,
                },
            }],
        );

        let stats = presenter.present(&draw_list).unwrap();

        assert_eq!(stats.surface_size, (32, 16));
        assert_eq!(stats.draw_calls, 1);
        assert_eq!(stats.visible_command_count, 1);
        assert_eq!(stats.visible_draw_item_count, 1);
        assert_eq!(stats.batch_layer_count, 1);
        assert_eq!(stats.batch_dependency_count, 0);
        assert_eq!(stats.presented_frame_count, 1);
        assert_eq!(presenter.last_present_stats(), stats);
    }

    #[test]
    fn wgpu_ui_surface_presenter_resize_tracks_draw_list_size() {
        let mut presenter = WgpuUiSurfacePresenter::new_headless(1, 1);
        let draw_list = UiSurfaceDrawList::new((64, 48), None, Vec::new());

        let stats = presenter.present(&draw_list).unwrap();

        assert_eq!(presenter.descriptor().clamped_size(), (64, 48));
        assert_eq!(stats.surface_size, (64, 48));
    }

    #[test]
    fn wgpu_ui_surface_presenter_stats_skip_disjoint_patch_commands() {
        let mut presenter = WgpuUiSurfacePresenter::new_headless(100, 100);
        let draw_list = UiSurfaceDrawList::new(
            (100, 100),
            Some(UiSurfaceRect::new(50.0, 50.0, 10.0, 10.0)),
            vec![
                UiSurfaceCommand {
                    z_index: 0,
                    frame: UiSurfaceRect::new(0.0, 0.0, 10.0, 10.0),
                    clip: None,
                    kind: UiSurfaceCommandKind::Quad {
                        color: [255, 255, 255, 255],
                        corner_radius: 0.0,
                    },
                },
                UiSurfaceCommand {
                    z_index: 1,
                    frame: UiSurfaceRect::new(50.0, 50.0, 5.0, 5.0),
                    clip: None,
                    kind: UiSurfaceCommandKind::Image {
                        payload: UiSurfaceImagePayload {
                            resource_key: "viewport".to_string(),
                            width: 2,
                            height: 2,
                            upload_bytes: 16,
                            rgba: Some(vec![255; 16]),
                        },
                    },
                },
            ],
        );

        let stats = presenter.present(&draw_list).unwrap();

        assert_eq!(stats.draw_calls, 1);
        assert_eq!(stats.visible_command_count, 1);
        assert_eq!(stats.visible_draw_item_count, 1);
        assert_eq!(stats.batch_layer_count, 1);
        assert_eq!(stats.batch_dependency_count, 0);
        assert_eq!(stats.image_count, 1);
        assert_eq!(stats.image_upload_bytes, 16);
    }

    #[test]
    fn wgpu_ui_surface_presenter_stats_report_batched_draw_calls() {
        let mut presenter = WgpuUiSurfacePresenter::new_headless(100, 100);
        let draw_list = UiSurfaceDrawList::new(
            (100, 100),
            None,
            vec![
                UiSurfaceCommand {
                    z_index: 0,
                    frame: UiSurfaceRect::new(0.0, 0.0, 10.0, 10.0),
                    clip: None,
                    kind: UiSurfaceCommandKind::Quad {
                        color: [255, 0, 0, 255],
                        corner_radius: 0.0,
                    },
                },
                UiSurfaceCommand {
                    z_index: 1,
                    frame: UiSurfaceRect::new(20.0, 0.0, 10.0, 10.0),
                    clip: None,
                    kind: UiSurfaceCommandKind::Quad {
                        color: [0, 255, 0, 255],
                        corner_radius: 0.0,
                    },
                },
            ],
        );

        let stats = presenter.present(&draw_list).unwrap();

        assert_eq!(stats.visible_command_count, 2);
        assert_eq!(stats.visible_draw_item_count, 2);
        assert_eq!(stats.draw_calls, 1);
        assert_eq!(stats.batch_layer_count, 1);
        assert_eq!(stats.batch_dependency_count, 0);
    }

    #[test]
    fn wgpu_ui_surface_image_cache_prune_keeps_recent_entries() {
        let prune = image_cache_keys_to_prune(
            [("oldest", 1), ("recent", 10), ("middle", 5), ("newest", 20)].into_iter(),
            2,
        );

        assert_eq!(prune, vec!["oldest".to_string(), "middle".to_string()]);
    }

    #[test]
    fn wgpu_ui_surface_image_cache_prune_is_stable_for_ties() {
        let prune =
            image_cache_keys_to_prune([("b", 1), ("c", 1), ("a", 1), ("d", 2)].into_iter(), 2);

        assert_eq!(prune, vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn wgpu_ui_surface_image_cache_prune_is_noop_under_budget() {
        let prune = image_cache_keys_to_prune([("one", 1), ("two", 2)].into_iter(), 2);

        assert!(prune.is_empty());
    }
}
