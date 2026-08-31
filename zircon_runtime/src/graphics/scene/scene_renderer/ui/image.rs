use std::collections::HashMap;
use std::ops::Range;
use std::sync::{Arc, Weak};
use zr_rhi_wgpu::{WgpuBufferUpload, WgpuBufferUploadBatch};

use bytemuck::{Pod, Zeroable};
use zircon_runtime_interface::ui::layout::UiFrame;

use crate::core::math::UVec2;
use crate::core::resource::{
    ResourceId, ResourceManagementGenerationIdentity, ResourceReadinessGenerationIdentity,
};
use crate::graphics::scene::resources::{GpuTextureResource, ResourceStreamer};

use super::render::{PlannedScreenSpaceUi, ScreenSpaceUiScissor};

const SCREEN_SPACE_UI_IMAGE_SHADER: &str = include_str!("shaders/screen_space_ui_image.wgsl");
const SCREEN_SPACE_UI_IMAGE_MIN_VERTEX_BUFFER_CAPACITY_BYTES: u64 = 4 * 1024;
const SCREEN_SPACE_UI_IMAGE_BINDING_CACHE_IDLE_EPOCHS: u64 = 2;
const SCREEN_SPACE_UI_IMAGE_BINDING_CACHE_MAX_ENTRIES: usize = 512;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ScreenSpaceUiImageBatch {
    pub(super) texture: ResourceId,
    pub(super) frame: UiFrame,
    pub(super) clip_frame: Option<UiFrame>,
    pub(super) tint: [f32; 4],
}

pub(super) struct ScreenSpaceUiImageSystem {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    image_bindings: ScreenSpaceUiImageBindingCache,
    prepared_textures: ScreenSpaceUiImagePrepareTextureCache,
    image_segments: Vec<ScreenSpaceUiImageSegmentCache>,
}

pub(super) struct PreparedScreenSpaceUiImage {
    vertex_range: Range<u32>,
    dependency_index: usize,
    scissor: ScreenSpaceUiScissor,
}

#[derive(Default)]
struct ScreenSpaceUiImageSegmentCache {
    plan: Option<Weak<PlannedScreenSpaceUi>>,
    viewport_size: UVec2,
    images: Vec<PreparedScreenSpaceUiImage>,
    dependencies: Vec<ScreenSpaceUiImageTextureDependency>,
    image_vertices: ScreenSpaceUiImageVertexBuffer,
}

struct ScreenSpaceUiImageTextureDependency {
    requested: ResourceId,
    resolved_texture_id: Option<ResourceId>,
    resolution_is_current: bool,
    texture: Option<Arc<GpuTextureResource>>,
    binding_handle: Option<ScreenSpaceUiImageBindingHandle>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ScreenSpaceUiImageBindingHandle(usize);

struct ScreenSpaceUiImageBindingCache {
    next_prepare_epoch: u64,
    bindings: HashMap<usize, CachedScreenSpaceUiImageBinding>,
}

struct CachedScreenSpaceUiImageBinding {
    texture: Arc<GpuTextureResource>,
    bind_group: wgpu::BindGroup,
    last_prepare_epoch: u64,
}

#[derive(Default)]
struct ScreenSpaceUiImagePrepareTextureCache {
    management_generation: Option<ResourceManagementGenerationIdentity>,
    readiness_generation: Option<ResourceReadinessGenerationIdentity>,
    frame_prepare_epoch: Option<u64>,
    resolved_texture_ids: HashMap<ResourceId, Option<ResourceId>>,
}

#[derive(Default)]
struct ScreenSpaceUiImageVertexBuffer {
    buffer: Option<wgpu::Buffer>,
    capacity_bytes: u64,
    payload_hash: Option<[u8; 32]>,
    vertices: Vec<ScreenSpaceUiImageVertex>,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct ScreenSpaceUiImageVertex {
    position: [f32; 2],
    uv: [f32; 2],
    tint: [f32; 4],
}

impl ScreenSpaceUiImageVertex {
    fn layout() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 3] =
            wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32x4];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

impl ScreenSpaceUiImageBindingCache {
    fn begin_prepare(&mut self) -> u64 {
        self.next_prepare_epoch = self.next_prepare_epoch.wrapping_add(1).max(1);
        self.next_prepare_epoch
    }

    fn binding_handle_for(
        &mut self,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        texture: &Arc<GpuTextureResource>,
        prepare_epoch: u64,
    ) -> ScreenSpaceUiImageBindingHandle {
        let key = Arc::as_ptr(texture) as usize;
        if let Some(cached) = self.bindings.get_mut(&key) {
            if Arc::ptr_eq(&cached.texture, texture) {
                cached.last_prepare_epoch = prepare_epoch;
                return ScreenSpaceUiImageBindingHandle(key);
            }
        }

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-screen-space-ui-image-bind-group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(texture.view()),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(texture.sampler()),
                },
            ],
        });
        self.bindings.insert(
            key,
            CachedScreenSpaceUiImageBinding {
                texture: Arc::clone(texture),
                bind_group,
                last_prepare_epoch: prepare_epoch,
            },
        );
        ScreenSpaceUiImageBindingHandle(key)
    }

    fn bind_group(&self, handle: ScreenSpaceUiImageBindingHandle) -> Option<&wgpu::BindGroup> {
        self.bindings
            .get(&handle.0)
            .map(|binding| &binding.bind_group)
    }

    fn retain_prepare_epoch(&mut self, prepare_epoch: u64) {
        self.bindings.retain(|_, binding| {
            binding_cache_epoch_is_recent(prepare_epoch, binding.last_prepare_epoch)
        });
        while self.bindings.len() > SCREEN_SPACE_UI_IMAGE_BINDING_CACHE_MAX_ENTRIES {
            let Some(stale_key) = self
                .bindings
                .iter()
                .filter(|(_, binding)| {
                    binding_cache_entry_is_trimmable(prepare_epoch, binding.last_prepare_epoch)
                })
                .min_by_key(|(_, binding)| binding.last_prepare_epoch)
                .map(|(key, _)| *key)
            else {
                break;
            };
            self.bindings.remove(&stale_key);
        }
    }

    #[cfg(test)]
    fn clear(&mut self) {
        self.bindings = HashMap::new();
    }
}

impl ScreenSpaceUiImagePrepareTextureCache {
    fn begin_prepare(
        &mut self,
        management_generation: Option<ResourceManagementGenerationIdentity>,
        readiness_generation: Option<ResourceReadinessGenerationIdentity>,
        frame_prepare_epoch: Option<u64>,
    ) -> bool {
        if self.management_generation == management_generation
            && self.readiness_generation == readiness_generation
            && self.frame_prepare_epoch == frame_prepare_epoch
        {
            return false;
        }
        self.resolved_texture_ids.clear();
        self.management_generation = management_generation;
        self.readiness_generation = readiness_generation;
        self.frame_prepare_epoch = frame_prepare_epoch;
        true
    }

    fn reset(&mut self) {
        self.management_generation = None;
        self.readiness_generation = None;
        self.frame_prepare_epoch = None;
        self.resolved_texture_ids = HashMap::new();
    }

    fn resolved_texture_id_for(
        &mut self,
        streamer: &ResourceStreamer,
        requested: ResourceId,
    ) -> Option<ResourceId> {
        if streamer.last_ui_texture_prepare_receipt().is_some() {
            return streamer.prepared_ui_texture_id(requested);
        }
        *self
            .resolved_texture_ids
            .entry(requested)
            .or_insert_with(|| streamer.resolve_ui_texture_id(requested))
    }
}

impl ScreenSpaceUiImageVertexBuffer {
    fn clear_cpu_staging(&mut self) {
        self.vertices = Vec::new();
        self.payload_hash = None;
    }
}

impl ScreenSpaceUiImageSystem {
    pub(super) fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("zircon-screen-space-ui-image-bind-group-layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("zircon-screen-space-ui-image-pipeline-layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("zircon-screen-space-ui-image-shader"),
            source: wgpu::ShaderSource::Wgsl(SCREEN_SPACE_UI_IMAGE_SHADER.into()),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("zircon-screen-space-ui-image-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[ScreenSpaceUiImageVertex::layout()],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        Self {
            pipeline,
            bind_group_layout,
            image_bindings: ScreenSpaceUiImageBindingCache {
                next_prepare_epoch: 0,
                bindings: HashMap::new(),
            },
            prepared_textures: ScreenSpaceUiImagePrepareTextureCache::default(),
            image_segments: Vec::new(),
        }
    }

    pub(super) fn clear_frame_state(&mut self) {
        let prepare_epoch = self.image_bindings.begin_prepare();
        self.image_bindings.retain_prepare_epoch(prepare_epoch);
        self.prepared_textures.reset();
        for segment in &mut self.image_segments {
            segment.plan = None;
            segment.images.clear();
            segment.dependencies.clear();
            segment.image_vertices.clear_cpu_staging();
        }
    }

    pub(super) fn prepare(
        &mut self,
        device: &wgpu::Device,
        viewport_size: UVec2,
        render_segments: &[Arc<PlannedScreenSpaceUi>],
        streamer: Option<&ResourceStreamer>,
        uploads: &mut WgpuBufferUploadBatch,
        force_full_upload: bool,
    ) {
        let Some(streamer) = streamer else {
            self.clear_frame_state();
            return;
        };
        let prepare_epoch = self.image_bindings.begin_prepare();
        let viewport = UiFrame::new(
            0.0,
            0.0,
            viewport_size.x.max(1) as f32,
            viewport_size.y.max(1) as f32,
        );
        let bind_group_layout = &self.bind_group_layout;
        let image_bindings = &mut self.image_bindings;
        let prepared_textures = &mut self.prepared_textures;
        let image_segments = &mut self.image_segments;
        let texture_prepare_generation = streamer
            .last_ui_texture_prepare_receipt()
            .map(|receipt| {
                (
                    Some(receipt.management_generation().clone()),
                    Some(receipt.readiness_generation().clone()),
                    Some(receipt.frame_prepare_epoch()),
                )
            })
            .or_else(|| {
                streamer.asset_manager().ok().map(|manager| {
                    let projection = manager.resource_manager().projection_snapshot();
                    (
                        Some(projection.management_identity()),
                        Some(projection.readiness_identity()),
                        None,
                    )
                })
            })
            .unwrap_or((None, None, None));
        let texture_resolution_generation_changed = prepared_textures.begin_prepare(
            texture_prepare_generation.0,
            texture_prepare_generation.1,
            texture_prepare_generation.2,
        );
        if image_segments.len() < render_segments.len() {
            image_segments.resize_with(
                render_segments.len(),
                ScreenSpaceUiImageSegmentCache::default,
            );
        }

        let mut segment_plan_reuse_count = 0_usize;
        let mut image_batch_visit_count = 0_usize;
        let mut texture_dependency_check_count = 0_usize;
        for (plan, segment) in render_segments.iter().zip(image_segments.iter_mut()) {
            if !force_full_upload
                && screen_space_ui_image_segment_plan_reused(
                    segment.plan.as_ref(),
                    segment.viewport_size,
                    plan,
                    viewport_size,
                )
            {
                segment_plan_reuse_count = segment_plan_reuse_count.saturating_add(1);
            } else {
                image_batch_visit_count =
                    image_batch_visit_count.saturating_add(plan.image_batches().len());
                Self::rebuild_segment_geometry(
                    device,
                    viewport,
                    viewport_size,
                    plan,
                    segment,
                    uploads,
                    force_full_upload,
                );
            }
            texture_dependency_check_count =
                texture_dependency_check_count.saturating_add(Self::refresh_segment_dependencies(
                    device,
                    bind_group_layout,
                    streamer,
                    prepare_epoch,
                    image_bindings,
                    prepared_textures,
                    texture_resolution_generation_changed,
                    segment,
                ));
        }
        image_segments.truncate(render_segments.len());
        image_bindings.retain_prepare_epoch(prepare_epoch);
        crate::core::diagnostics::profiling::record_counter_batch(
            "runtime",
            &[
                (
                    "ui.screen_space_ui_image.segment_plan_reuse_count",
                    segment_plan_reuse_count as f64,
                ),
                (
                    "ui.screen_space_ui_image.batch_visit_count",
                    image_batch_visit_count as f64,
                ),
                (
                    "ui.screen_space_ui_image.texture_dependency_check_count",
                    texture_dependency_check_count as f64,
                ),
            ],
        );
    }

    fn rebuild_segment_geometry(
        device: &wgpu::Device,
        viewport: UiFrame,
        viewport_size: UVec2,
        plan: &Arc<PlannedScreenSpaceUi>,
        segment: &mut ScreenSpaceUiImageSegmentCache,
        uploads: &mut WgpuBufferUploadBatch,
        force_full_upload: bool,
    ) {
        segment.images.clear();
        segment.dependencies.clear();
        segment.image_vertices.vertices.clear();
        let mut dependency_indices = HashMap::new();
        for batch in plan.image_batches() {
            let Some(image) = Self::prepare_batch_geometry(
                viewport,
                batch,
                &mut dependency_indices,
                &mut segment.dependencies,
                &mut segment.image_vertices.vertices,
            ) else {
                continue;
            };
            segment.images.push(image);
        }
        if image_cpu_staging_should_reset(segment.images.len()) {
            segment.image_vertices.clear_cpu_staging();
        } else {
            write_screen_space_ui_image_vertex_buffer(
                device,
                &mut segment.image_vertices,
                uploads,
                force_full_upload,
            );
        }
        segment.plan = Some(Arc::downgrade(plan));
        segment.viewport_size = viewport_size;
    }

    fn prepare_batch_geometry(
        viewport: UiFrame,
        batch: &ScreenSpaceUiImageBatch,
        dependency_indices: &mut HashMap<ResourceId, usize>,
        dependencies: &mut Vec<ScreenSpaceUiImageTextureDependency>,
        vertices: &mut Vec<ScreenSpaceUiImageVertex>,
    ) -> Option<PreparedScreenSpaceUiImage> {
        if batch.frame.width <= 0.0 || batch.frame.height <= 0.0 {
            return None;
        }
        let scissor = image_batch_scissor(batch.frame, viewport, batch.clip_frame)?;
        let dependency_index = *dependency_indices.entry(batch.texture).or_insert_with(|| {
            let dependency_index = dependencies.len();
            dependencies.push(ScreenSpaceUiImageTextureDependency {
                requested: batch.texture,
                resolved_texture_id: None,
                resolution_is_current: false,
                texture: None,
                binding_handle: None,
            });
            dependency_index
        });
        let vertex_start = u32::try_from(vertices.len()).ok()?;
        vertices.extend(image_vertices(batch.frame, viewport, batch.tint));
        let vertex_end = u32::try_from(vertices.len()).ok()?;
        Some(PreparedScreenSpaceUiImage {
            vertex_range: vertex_start..vertex_end,
            dependency_index,
            scissor,
        })
    }

    fn refresh_segment_dependencies(
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        streamer: &ResourceStreamer,
        prepare_epoch: u64,
        image_bindings: &mut ScreenSpaceUiImageBindingCache,
        prepared_textures: &mut ScreenSpaceUiImagePrepareTextureCache,
        texture_resolution_generation_changed: bool,
        segment: &mut ScreenSpaceUiImageSegmentCache,
    ) -> usize {
        for dependency in &mut segment.dependencies {
            if texture_resolution_generation_changed || !dependency.resolution_is_current {
                dependency.resolved_texture_id =
                    prepared_textures.resolved_texture_id_for(streamer, dependency.requested);
                dependency.resolution_is_current = true;
            }
            let texture = streamer.ui_texture_ref(dependency.resolved_texture_id);
            let binding_handle = image_bindings.binding_handle_for(
                device,
                bind_group_layout,
                texture,
                prepare_epoch,
            );
            if !screen_space_ui_image_texture_dependency_is_current(
                dependency.texture.as_ref(),
                texture,
            ) {
                dependency.texture = Some(Arc::clone(texture));
            }
            dependency.binding_handle = Some(binding_handle);
        }
        segment.dependencies.len()
    }

    pub(super) fn render<'pass>(&'pass self, pass: &mut wgpu::RenderPass<'pass>) {
        let mut pipeline_is_bound = false;
        for segment in &self.image_segments {
            if segment.images.is_empty() {
                continue;
            }
            let Some(vertex_buffer) = segment.image_vertices.buffer.as_ref() else {
                continue;
            };
            if !pipeline_is_bound {
                pass.set_pipeline(&self.pipeline);
                pipeline_is_bound = true;
            }
            pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            for image in &segment.images {
                let Some(binding_handle) = segment
                    .dependencies
                    .get(image.dependency_index)
                    .and_then(|dependency| dependency.binding_handle)
                else {
                    continue;
                };
                let Some(bind_group) = self.image_bindings.bind_group(binding_handle) else {
                    continue;
                };
                pass.set_scissor_rect(
                    image.scissor.x,
                    image.scissor.y,
                    image.scissor.width,
                    image.scissor.height,
                );
                pass.set_bind_group(0, bind_group, &[]);
                pass.draw(image.vertex_range.clone(), 0..1);
            }
        }
    }
}

fn screen_space_ui_image_segment_plan_reused(
    current: Option<&Weak<PlannedScreenSpaceUi>>,
    current_viewport_size: UVec2,
    next: &Arc<PlannedScreenSpaceUi>,
    next_viewport_size: UVec2,
) -> bool {
    current_viewport_size == next_viewport_size
        && current.is_some_and(|current| std::ptr::eq(current.as_ptr(), Arc::as_ptr(next)))
}

fn screen_space_ui_image_texture_dependency_is_current<T>(
    current: Option<&Arc<T>>,
    next: &Arc<T>,
) -> bool {
    current.is_some_and(|current| Arc::ptr_eq(current, next))
}

fn image_batch_scissor(
    frame: UiFrame,
    viewport: UiFrame,
    clip_frame: Option<UiFrame>,
) -> Option<ScreenSpaceUiScissor> {
    super::render::clipped_scissor(
        frame,
        clip_frame,
        viewport,
        super::render::frame_to_scissor(viewport)?,
    )
}

fn write_screen_space_ui_image_vertex_buffer(
    device: &wgpu::Device,
    image_vertices: &mut ScreenSpaceUiImageVertexBuffer,
    uploads: &mut WgpuBufferUploadBatch,
    force_full_upload: bool,
) {
    if image_vertices.vertices.is_empty() {
        return;
    }

    let vertex_bytes = bytemuck::cast_slice(image_vertices.vertices.as_slice());
    let required_byte_len = vertex_bytes.len();
    let requires_reallocation = image_vertices.buffer.is_none()
        || image_vertex_buffer_requires_reallocation(
            image_vertices.capacity_bytes,
            required_byte_len,
        );
    if requires_reallocation {
        let capacity_bytes = image_vertex_buffer_capacity(required_byte_len);
        image_vertices.buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-screen-space-ui-image-vertices"),
            size: capacity_bytes,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));
        image_vertices.capacity_bytes = capacity_bytes;
    }
    let payload_hash = *blake3::hash(vertex_bytes).as_bytes();
    let write_required = image_vertex_buffer_write_required(
        requires_reallocation || force_full_upload,
        image_vertices.payload_hash,
        payload_hash,
    );
    if write_required {
        if let Some(vertex_buffer) = image_vertices.buffer.as_ref() {
            uploads.push(WgpuBufferUpload::from_bytes(
                vertex_buffer.clone(),
                0,
                vertex_bytes,
            ));
            image_vertices.payload_hash = Some(payload_hash);
        }
    }
}

const fn image_cpu_staging_should_reset(image_count: usize) -> bool {
    image_count == 0
}

fn binding_cache_epoch_is_recent(current_epoch: u64, last_prepare_epoch: u64) -> bool {
    current_epoch >= last_prepare_epoch
        && current_epoch - last_prepare_epoch <= SCREEN_SPACE_UI_IMAGE_BINDING_CACHE_IDLE_EPOCHS
}

fn binding_cache_entry_is_trimmable(current_epoch: u64, last_prepare_epoch: u64) -> bool {
    last_prepare_epoch != current_epoch
}

fn image_vertex_buffer_capacity(required_byte_len: usize) -> u64 {
    let required_byte_len =
        (required_byte_len as u64).max(SCREEN_SPACE_UI_IMAGE_MIN_VERTEX_BUFFER_CAPACITY_BYTES);
    required_byte_len
        .checked_next_power_of_two()
        .unwrap_or(required_byte_len)
}

fn image_vertex_buffer_requires_reallocation(
    capacity_bytes: u64,
    required_byte_len: usize,
) -> bool {
    capacity_bytes < required_byte_len as u64
}

fn image_vertex_buffer_write_required(
    requires_reallocation: bool,
    current_payload_hash: Option<[u8; 32]>,
    next_payload_hash: [u8; 32],
) -> bool {
    requires_reallocation || current_payload_hash != Some(next_payload_hash)
}

fn image_vertices(
    frame: UiFrame,
    viewport: UiFrame,
    tint: [f32; 4],
) -> [ScreenSpaceUiImageVertex; 6] {
    let x0 = (frame.x / viewport.width.max(1.0)) * 2.0 - 1.0;
    let x1 = (frame.right() / viewport.width.max(1.0)) * 2.0 - 1.0;
    let y0 = 1.0 - (frame.y / viewport.height.max(1.0)) * 2.0;
    let y1 = 1.0 - (frame.bottom() / viewport.height.max(1.0)) * 2.0;
    [
        ScreenSpaceUiImageVertex {
            position: [x0, y0],
            uv: [0.0, 0.0],
            tint,
        },
        ScreenSpaceUiImageVertex {
            position: [x1, y0],
            uv: [1.0, 0.0],
            tint,
        },
        ScreenSpaceUiImageVertex {
            position: [x1, y1],
            uv: [1.0, 1.0],
            tint,
        },
        ScreenSpaceUiImageVertex {
            position: [x0, y0],
            uv: [0.0, 0.0],
            tint,
        },
        ScreenSpaceUiImageVertex {
            position: [x1, y1],
            uv: [1.0, 1.0],
            tint,
        },
        ScreenSpaceUiImageVertex {
            position: [x0, y1],
            uv: [0.0, 1.0],
            tint,
        },
    ]
}

#[cfg(test)]
mod tests;
