use std::collections::HashMap;
use std::ops::Range;
use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use zircon_runtime_interface::ui::layout::UiFrame;

use crate::core::math::UVec2;
use crate::core::resource::ResourceId;
use crate::graphics::scene::resources::{GpuTextureResource, ResourceStreamer};

use super::render::ScreenSpaceUiScissor;

const SCREEN_SPACE_UI_IMAGE_SHADER: &str = include_str!("shaders/screen_space_ui_image.wgsl");
const SCREEN_SPACE_UI_IMAGE_MIN_VERTEX_BUFFER_CAPACITY_BYTES: u64 = 4 * 1024;

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
    image_vertices: ScreenSpaceUiImageVertexBuffer,
}

pub(super) struct PreparedScreenSpaceUiImage {
    vertex_range: Range<u32>,
    binding_handle: ScreenSpaceUiImageBindingHandle,
    scissor: ScreenSpaceUiScissor,
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
        self.bindings
            .retain(|_, binding| binding.last_prepare_epoch == prepare_epoch);
        if self.bindings.is_empty() {
            self.clear();
        }
    }

    fn clear(&mut self) {
        self.bindings = HashMap::new();
    }
}

impl ScreenSpaceUiImagePrepareTextureCache {
    fn clear(&mut self) {
        self.resolved_texture_ids.clear();
    }

    fn reset(&mut self) {
        self.resolved_texture_ids = HashMap::new();
    }

    fn texture_for<'a>(
        &mut self,
        streamer: &'a ResourceStreamer,
        requested: ResourceId,
    ) -> &'a Arc<GpuTextureResource> {
        let resolved_texture_id = *self
            .resolved_texture_ids
            .entry(requested)
            .or_insert_with(|| streamer.resolve_ui_texture_id(requested));
        streamer.ui_texture_ref(resolved_texture_id)
    }
}

impl ScreenSpaceUiImageVertexBuffer {
    fn clear_cpu_staging(&mut self) {
        self.vertices = Vec::new();
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
            image_vertices: ScreenSpaceUiImageVertexBuffer::default(),
        }
    }

    pub(super) fn clear_frame_state(&mut self) {
        self.image_bindings.clear();
        self.prepared_textures.reset();
        self.image_vertices.clear_cpu_staging();
    }

    pub(super) fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        viewport_size: UVec2,
        batches: &[ScreenSpaceUiImageBatch],
        streamer: Option<&ResourceStreamer>,
    ) -> Vec<PreparedScreenSpaceUiImage> {
        let Some(streamer) = streamer else {
            self.clear_frame_state();
            return Vec::new();
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
        let image_vertices = &mut self.image_vertices;
        prepared_textures.clear();
        image_vertices.vertices.clear();
        let images: Vec<PreparedScreenSpaceUiImage> = batches
            .iter()
            .filter_map(|batch| {
                Self::prepare_batch(
                    device,
                    bind_group_layout,
                    viewport,
                    batch,
                    streamer,
                    prepare_epoch,
                    image_bindings,
                    prepared_textures,
                    &mut image_vertices.vertices,
                )
            })
            .collect();
        if image_cpu_staging_should_reset(images.len()) {
            prepared_textures.reset();
            image_vertices.clear_cpu_staging();
        } else {
            write_screen_space_ui_image_vertex_buffer(device, queue, image_vertices);
        }
        image_bindings.retain_prepare_epoch(prepare_epoch);
        images
    }

    fn prepare_batch(
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        viewport: UiFrame,
        batch: &ScreenSpaceUiImageBatch,
        streamer: &ResourceStreamer,
        prepare_epoch: u64,
        image_bindings: &mut ScreenSpaceUiImageBindingCache,
        prepared_textures: &mut ScreenSpaceUiImagePrepareTextureCache,
        vertices: &mut Vec<ScreenSpaceUiImageVertex>,
    ) -> Option<PreparedScreenSpaceUiImage> {
        if batch.frame.width <= 0.0 || batch.frame.height <= 0.0 {
            return None;
        }
        let scissor = image_batch_scissor(batch.frame, viewport, batch.clip_frame)?;
        let texture = prepared_textures.texture_for(streamer, batch.texture);
        let binding_handle =
            image_bindings.binding_handle_for(device, bind_group_layout, texture, prepare_epoch);
        let vertex_start = u32::try_from(vertices.len()).ok()?;
        vertices.extend(image_vertices(batch.frame, viewport, batch.tint));
        let vertex_end = u32::try_from(vertices.len()).ok()?;
        Some(PreparedScreenSpaceUiImage {
            vertex_range: vertex_start..vertex_end,
            binding_handle,
            scissor,
        })
    }

    pub(super) fn render<'pass>(
        &'pass self,
        pass: &mut wgpu::RenderPass<'pass>,
        images: &'pass [PreparedScreenSpaceUiImage],
    ) {
        if images.is_empty() {
            return;
        }
        let Some(vertex_buffer) = self.image_vertices.buffer.as_ref() else {
            return;
        };
        pass.set_pipeline(&self.pipeline);
        pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        for image in images {
            let Some(bind_group) = self.image_bindings.bind_group(image.binding_handle) else {
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
    queue: &wgpu::Queue,
    image_vertices: &mut ScreenSpaceUiImageVertexBuffer,
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
        requires_reallocation,
        image_vertices.payload_hash,
        payload_hash,
    );
    if write_required {
        if let Some(vertex_buffer) = image_vertices.buffer.as_ref() {
            queue.write_buffer(vertex_buffer, 0, vertex_bytes);
            image_vertices.payload_hash = Some(payload_hash);
        }
    }
}

const fn image_cpu_staging_should_reset(image_count: usize) -> bool {
    image_count == 0
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
