use wgpu::util::DeviceExt;

use crate::graphics::scene::scene_renderer::overlay::PreparedOverlayBuffers;
use crate::graphics::types::{ViewportRenderFrame, ViewportRenderRegion};

use super::viewport_overlay_renderer::ViewportOverlayRenderer;

const SHADER: &str = include_str!("depth_reconstruction.wgsl");

pub(super) struct OverlayDepthReconstruction {
    layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,
}

impl OverlayDepthReconstruction {
    /// 建立源深度复制管线；录制时先清除整个输出深度附件，再在输出区域以 Always 比较写入复制结果。
    pub(super) fn new(device: &wgpu::Device) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("zircon-overlay-depth-reconstruction-layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("zircon-overlay-depth-reconstruction-pipeline-layout"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("zircon-overlay-depth-reconstruction-shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("zircon-overlay-depth-reconstruction-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Always),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[],
            }),
            multiview_mask: None,
            cache: None,
        });
        Self { layout, pipeline }
    }

    fn record(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        source_view: &wgpu::TextureView,
        output_view: &wgpu::TextureView,
        source_region: ViewportRenderRegion,
        output_region: ViewportRenderRegion,
    ) {
        // 空区域不能安全地做 `source_size - 1` 的像素钳制，也不会产生可见 overlay。
        let source_origin = source_region.physical_position();
        let source_size = source_region.physical_size();
        let output_origin = output_region.physical_position();
        let output_size = output_region.physical_size();
        if source_region.is_empty() || output_region.is_empty() {
            return;
        }
        let params = [
            source_origin.x,
            source_origin.y,
            source_size.x,
            source_size.y,
            output_origin.x,
            output_origin.y,
            output_size.x,
            output_size.y,
        ];
        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("zircon-overlay-depth-region"),
            contents: bytemuck::cast_slice(&params),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-overlay-depth-reconstruction-bind-group"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(source_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: params_buffer.as_entire_binding(),
                },
            ],
        });
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("zircon-overlay-depth-reconstruction"),
            color_attachments: &[],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: output_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        if output_region.apply_physical_to_render_pass(&mut pass) {
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
    }
}

impl ViewportOverlayRenderer {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record_overlay_depth_reconstruction(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        source_view: &wgpu::TextureView,
        output_view: &wgpu::TextureView,
        frame: &ViewportRenderFrame,
        prepared: &PreparedOverlayBuffers,
        source_region: ViewportRenderRegion,
        output_region: ViewportRenderRegion,
    ) {
        // Graph pass 的输入/输出分别是 SCENE_DEPTH 与 VIEWPORT_OVERLAY_DEPTH；无可绘制交互项时跳过全屏复制。
        let Some(interaction) = self.interaction_overlays.as_ref() else {
            return;
        };
        let has_drawable = prepared.selection_buffer.is_some()
            || prepared.wireframe_buffer.is_some()
            || prepared.scene_gizmo.line_buffer.is_some()
            || !prepared.scene_gizmo.icon_draws.is_empty()
            || prepared.handle_buffer.is_some()
            || frame
                .scene
                .overlays
                .grid
                .as_ref()
                .is_some_and(|grid| grid.visible);
        if !has_drawable {
            return;
        }
        interaction.depth_reconstruction.record(
            device,
            encoder,
            source_view,
            output_view,
            source_region,
            output_region,
        );
    }
}
