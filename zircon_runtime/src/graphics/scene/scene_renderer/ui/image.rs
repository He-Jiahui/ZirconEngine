use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;
use zircon_runtime_interface::ui::layout::UiFrame;

use crate::core::math::UVec2;
use crate::core::resource::ResourceId;
use crate::graphics::scene::resources::{GpuTextureResource, ResourceStreamer};

use super::render::ScreenSpaceUiScissor;

const SCREEN_SPACE_UI_IMAGE_SHADER: &str = include_str!("shaders/screen_space_ui_image.wgsl");

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
}

pub(super) struct PreparedScreenSpaceUiImage {
    vertex_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    scissor: ScreenSpaceUiScissor,
    _texture: Arc<GpuTextureResource>,
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
        }
    }

    pub(super) fn prepare(
        &self,
        device: &wgpu::Device,
        viewport_size: UVec2,
        batches: &[ScreenSpaceUiImageBatch],
        streamer: Option<&ResourceStreamer>,
    ) -> Vec<PreparedScreenSpaceUiImage> {
        let Some(streamer) = streamer else {
            return Vec::new();
        };
        let viewport = UiFrame::new(
            0.0,
            0.0,
            viewport_size.x.max(1) as f32,
            viewport_size.y.max(1) as f32,
        );
        batches
            .iter()
            .filter_map(|batch| self.prepare_batch(device, viewport, batch, streamer))
            .collect()
    }

    fn prepare_batch(
        &self,
        device: &wgpu::Device,
        viewport: UiFrame,
        batch: &ScreenSpaceUiImageBatch,
        streamer: &ResourceStreamer,
    ) -> Option<PreparedScreenSpaceUiImage> {
        if batch.frame.width <= 0.0
            || batch.frame.height <= 0.0
            || viewport.intersection(batch.frame).is_none()
        {
            return None;
        }
        let clip = batch
            .clip_frame
            .and_then(|clip| viewport.intersection(clip))
            .unwrap_or(viewport);
        let scissor = super::render::frame_to_scissor(clip)?;
        let texture = streamer.ui_texture(batch.texture);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-screen-space-ui-image-bind-group"),
            layout: &self.bind_group_layout,
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
        let vertices = image_vertices(batch.frame, viewport, batch.tint);
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("zircon-screen-space-ui-image-vertices"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        Some(PreparedScreenSpaceUiImage {
            vertex_buffer,
            bind_group,
            scissor,
            _texture: texture,
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
        pass.set_pipeline(&self.pipeline);
        for image in images {
            pass.set_scissor_rect(
                image.scissor.x,
                image.scissor.y,
                image.scissor.width,
                image.scissor.height,
            );
            pass.set_bind_group(0, &image.bind_group, &[]);
            pass.set_vertex_buffer(0, image.vertex_buffer.slice(..));
            pass.draw(0..6, 0..1);
        }
    }
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
