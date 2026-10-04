use std::collections::HashMap;
use std::ops::Range;
use std::sync::{Arc, Weak};
use zr_rhi_wgpu::WgpuBufferUploadBatch;

use bytemuck::{Pod, Zeroable};
use zircon_runtime_interface::ui::layout::UiFrame;

use crate::core::math::UVec2;
use crate::core::resource::{
    ResourceId, ResourceManagementGenerationIdentity, ResourceReadinessGenerationIdentity,
};
use crate::graphics::scene::resources::{GpuTextureResource, ResourceStreamer};

use super::render::{PlannedScreenSpaceUi, PreparedScreenSpaceUi, ScreenSpaceUiScissor};

mod geometry;

use geometry::{
    binding_cache_entry_is_trimmable, binding_cache_epoch_is_recent, image_batch_scissor,
    image_cpu_staging_should_reset, image_vertex_buffer_capacity,
    image_vertex_buffer_requires_reallocation, image_vertex_buffer_write_required, image_vertices,
    screen_space_ui_image_segment_plan_reused, screen_space_ui_image_texture_dependency_is_current,
    write_screen_space_ui_image_vertex_buffer,
};

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
    frame_generation: Option<u64>,
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
    binding_product: Option<Arc<ScreenSpaceUiImageBindingProduct>>,
}

struct ScreenSpaceUiImageBindingCache {
    next_prepare_epoch: u64,
    bindings: HashMap<usize, CachedScreenSpaceUiImageBinding>,
}

struct CachedScreenSpaceUiImageBinding {
    product: Arc<ScreenSpaceUiImageBindingProduct>,
    last_prepare_epoch: u64,
}

struct ScreenSpaceUiImageBindingProduct {
    texture: Arc<GpuTextureResource>,
    bind_group: Arc<wgpu::BindGroup>,
}

#[derive(Default)]
struct ScreenSpaceUiImagePrepareTextureCache {
    management_generation: Option<ResourceManagementGenerationIdentity>,
    readiness_generation: Option<ResourceReadinessGenerationIdentity>,
    binding_product_generation: Option<u64>,
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

    fn binding_product_for(
        &mut self,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        texture: &Arc<GpuTextureResource>,
        prepare_epoch: u64,
    ) -> Arc<ScreenSpaceUiImageBindingProduct> {
        let key = Arc::as_ptr(texture) as usize;
        if let Some(cached) = self.bindings.get_mut(&key) {
            if Arc::ptr_eq(&cached.product.texture, texture) {
                cached.last_prepare_epoch = prepare_epoch;
                return Arc::clone(&cached.product);
            }
        }

        let bind_group = Arc::new(device.create_bind_group(&wgpu::BindGroupDescriptor {
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
        }));
        let product = Arc::new(ScreenSpaceUiImageBindingProduct {
            texture: Arc::clone(texture),
            bind_group,
        });
        self.bindings.insert(
            key,
            CachedScreenSpaceUiImageBinding {
                product: Arc::clone(&product),
                last_prepare_epoch: prepare_epoch,
            },
        );
        self.trim_unpinned_overflow(prepare_epoch);
        product
    }

    fn retain_prepare_epoch(&mut self, prepare_epoch: u64) {
        // Segment-held products stay pinned; only idle cache entries participate in age/size trim.
        self.bindings.retain(|_, binding| {
            Arc::strong_count(&binding.product) > 1
                || binding_cache_epoch_is_recent(prepare_epoch, binding.last_prepare_epoch)
        });
        self.trim_unpinned_overflow(prepare_epoch);
    }

    fn trim_unpinned_overflow(&mut self, prepare_epoch: u64) {
        while self.bindings.len() > SCREEN_SPACE_UI_IMAGE_BINDING_CACHE_MAX_ENTRIES {
            let Some(stale_key) = self
                .bindings
                .iter()
                .filter(|(_, binding)| {
                    Arc::strong_count(&binding.product) == 1
                        && binding_cache_entry_is_trimmable(
                            prepare_epoch,
                            binding.last_prepare_epoch,
                        )
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
    fn generation_matches(
        &self,
        management_generation: Option<&ResourceManagementGenerationIdentity>,
        readiness_generation: Option<&ResourceReadinessGenerationIdentity>,
        binding_product_generation: Option<u64>,
    ) -> bool {
        self.management_generation.as_ref() == management_generation
            && self.readiness_generation.as_ref() == readiness_generation
            && self.binding_product_generation == binding_product_generation
    }

    fn begin_prepare(
        &mut self,
        management_generation: Option<ResourceManagementGenerationIdentity>,
        readiness_generation: Option<ResourceReadinessGenerationIdentity>,
        binding_product_generation: Option<u64>,
    ) -> bool {
        if self.management_generation == management_generation
            && self.readiness_generation == readiness_generation
            && self.binding_product_generation == binding_product_generation
        {
            return false;
        }
        self.resolved_texture_ids.clear();
        self.management_generation = management_generation;
        self.readiness_generation = readiness_generation;
        self.binding_product_generation = binding_product_generation;
        true
    }

    fn reset(&mut self) {
        self.management_generation = None;
        self.readiness_generation = None;
        self.binding_product_generation = None;
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
            frame_generation: None,
        }
    }

    pub(super) fn clear_frame_state(&mut self) {
        let prepare_epoch = self.image_bindings.begin_prepare();
        self.image_bindings.retain_prepare_epoch(prepare_epoch);
        self.prepared_textures.reset();
        self.frame_generation = None;
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
        prepared: &Arc<PreparedScreenSpaceUi>,
        streamer: Option<&ResourceStreamer>,
        uploads: &mut WgpuBufferUploadBatch,
        force_full_upload: bool,
    ) {
        let Some(streamer) = streamer else {
            self.clear_frame_state();
            return;
        };
        let render_segments = prepared.render_segments();
        let texture_prepare_generation = streamer
            .last_ui_texture_prepare_receipt()
            .map(|receipt| (None, None, Some(receipt.binding_product_generation())))
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
        let frame_generation_matches = self.frame_generation == Some(prepared.generation());
        let texture_generation_matches = self.prepared_textures.generation_matches(
            texture_prepare_generation.0.as_ref(),
            texture_prepare_generation.1.as_ref(),
            texture_prepare_generation.2,
        );
        if frame_generation_matches && !force_full_upload && texture_generation_matches {
            return;
        }

        let prepare_epoch = self.image_bindings.begin_prepare();
        self.image_bindings.retain_prepare_epoch(prepare_epoch);
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
        let journal = prepared.change_journal();
        let journal_applies =
            !journal.is_full_rebuild() && journal.base_generation() == self.frame_generation;
        let full_geometry_rebuild =
            force_full_upload || (!frame_generation_matches && !journal_applies);
        if full_geometry_rebuild || texture_resolution_generation_changed {
            for (plan, segment) in render_segments.iter().zip(image_segments.iter_mut()) {
                let (reused, dependency_checks) = Self::prepare_image_segment(
                    device,
                    viewport,
                    viewport_size,
                    plan,
                    segment,
                    uploads,
                    force_full_upload,
                    full_geometry_rebuild,
                    bind_group_layout,
                    streamer,
                    prepare_epoch,
                    image_bindings,
                    prepared_textures,
                    texture_resolution_generation_changed,
                );
                if reused {
                    segment_plan_reuse_count = segment_plan_reuse_count.saturating_add(1);
                } else {
                    image_batch_visit_count =
                        image_batch_visit_count.saturating_add(plan.image_batches().len());
                }
                texture_dependency_check_count =
                    texture_dependency_check_count.saturating_add(dependency_checks);
            }
        } else if journal_applies {
            for &index in journal.changed_segment_indices() {
                let Some(plan) = render_segments.get(index) else {
                    continue;
                };
                let Some(segment) = image_segments.get_mut(index) else {
                    continue;
                };
                let (reused, dependency_checks) = Self::prepare_image_segment(
                    device,
                    viewport,
                    viewport_size,
                    plan,
                    segment,
                    uploads,
                    force_full_upload,
                    true,
                    bind_group_layout,
                    streamer,
                    prepare_epoch,
                    image_bindings,
                    prepared_textures,
                    texture_resolution_generation_changed,
                );
                if reused {
                    segment_plan_reuse_count = segment_plan_reuse_count.saturating_add(1);
                } else {
                    image_batch_visit_count =
                        image_batch_visit_count.saturating_add(plan.image_batches().len());
                }
                texture_dependency_check_count =
                    texture_dependency_check_count.saturating_add(dependency_checks);
            }
        } else {
            for (plan, segment) in render_segments.iter().zip(image_segments.iter_mut()) {
                let (reused, dependency_checks) = Self::prepare_image_segment(
                    device,
                    viewport,
                    viewport_size,
                    plan,
                    segment,
                    uploads,
                    force_full_upload,
                    true,
                    bind_group_layout,
                    streamer,
                    prepare_epoch,
                    image_bindings,
                    prepared_textures,
                    texture_resolution_generation_changed,
                );
                if reused {
                    segment_plan_reuse_count = segment_plan_reuse_count.saturating_add(1);
                } else {
                    image_batch_visit_count =
                        image_batch_visit_count.saturating_add(plan.image_batches().len());
                }
                texture_dependency_check_count =
                    texture_dependency_check_count.saturating_add(dependency_checks);
            }
        }
        image_segments.truncate(render_segments.len());
        self.frame_generation = Some(prepared.generation());
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

    #[allow(clippy::too_many_arguments)]
    fn prepare_image_segment(
        device: &wgpu::Device,
        viewport: UiFrame,
        viewport_size: UVec2,
        plan: &Arc<PlannedScreenSpaceUi>,
        segment: &mut ScreenSpaceUiImageSegmentCache,
        uploads: &mut WgpuBufferUploadBatch,
        force_full_upload: bool,
        rebuild_geometry: bool,
        bind_group_layout: &wgpu::BindGroupLayout,
        streamer: &ResourceStreamer,
        prepare_epoch: u64,
        image_bindings: &mut ScreenSpaceUiImageBindingCache,
        prepared_textures: &mut ScreenSpaceUiImagePrepareTextureCache,
        texture_resolution_generation_changed: bool,
    ) -> (bool, usize) {
        let reused = !rebuild_geometry
            && !force_full_upload
            && screen_space_ui_image_segment_plan_reused(
                segment.plan.as_ref(),
                segment.viewport_size,
                plan,
                viewport_size,
            );
        if !reused {
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
        let dependency_checks = Self::refresh_segment_dependencies(
            device,
            bind_group_layout,
            streamer,
            prepare_epoch,
            image_bindings,
            prepared_textures,
            texture_resolution_generation_changed,
            segment,
        );
        (reused, dependency_checks)
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
                binding_product: None,
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
            let binding_product = image_bindings.binding_product_for(
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
            dependency.binding_product = Some(binding_product);
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
                let Some(binding_product) = segment
                    .dependencies
                    .get(image.dependency_index)
                    .and_then(|dependency| dependency.binding_product.as_ref())
                else {
                    continue;
                };
                pass.set_scissor_rect(
                    image.scissor.x,
                    image.scissor.y,
                    image.scissor.width,
                    image.scissor.height,
                );
                pass.set_bind_group(0, binding_product.bind_group.as_ref(), &[]);
                pass.draw(image.vertex_range.clone(), 0..1);
            }
        }
    }
}

#[cfg(test)]
#[path = "image/tests/cases.rs"]
mod tests;
