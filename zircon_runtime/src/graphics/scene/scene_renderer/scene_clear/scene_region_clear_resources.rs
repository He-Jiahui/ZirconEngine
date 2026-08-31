use crate::core::math::Vec4;
use crate::graphics::types::{
    ViewportRenderFrame, ViewportRenderRegion, ViewportSceneClearPlan, ViewportSceneColorClear,
};
use zr_rhi_wgpu::{WgpuBufferUpload, WgpuBufferUploadBatch};

use super::scene_region_clear_color_uniform::SceneRegionClearColorUniform;
use super::scene_region_clear_shader::SCENE_REGION_CLEAR_SHADER;

pub(crate) struct SceneRegionClearResources {
    color_buffer: wgpu::Buffer,
    color_bind_group: wgpu::BindGroup,
    color_pipeline: wgpu::RenderPipeline,
    color_depth_pipeline: wgpu::RenderPipeline,
    depth_pipeline: wgpu::RenderPipeline,
}

impl SceneRegionClearResources {
    pub(crate) fn new(
        device: &wgpu::Device,
        color_format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
    ) -> Self {
        let color_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-scene-region-clear-color"),
            size: std::mem::size_of::<SceneRegionClearColorUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let color_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("zircon-scene-region-clear-bind-group-layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let color_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-scene-region-clear-bind-group"),
            layout: &color_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: color_buffer.as_entire_binding(),
            }],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("zircon-scene-region-clear-shader"),
            source: wgpu::ShaderSource::Wgsl(SCENE_REGION_CLEAR_SHADER.into()),
        });
        let color_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("zircon-scene-region-clear-color-layout"),
                bind_group_layouts: &[Some(&color_bind_group_layout)],
                immediate_size: 0,
            });
        let depth_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("zircon-scene-region-clear-depth-layout"),
                bind_group_layouts: &[],
                immediate_size: 0,
            });
        let color_pipeline = create_color_pipeline(
            device,
            &shader,
            &color_pipeline_layout,
            color_format,
            None,
            "zircon-scene-region-clear-color-pipeline",
        );
        let color_depth_pipeline = create_color_pipeline(
            device,
            &shader,
            &color_pipeline_layout,
            color_format,
            Some(depth_format),
            "zircon-scene-region-clear-color-depth-pipeline",
        );
        let depth_pipeline = create_depth_pipeline(
            device,
            &shader,
            &depth_pipeline_layout,
            depth_format,
            "zircon-scene-region-clear-depth-pipeline",
        );

        Self {
            color_buffer,
            color_bind_group,
            color_pipeline,
            color_depth_pipeline,
            depth_pipeline,
        }
    }

    pub(crate) fn record_frame_clear(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        scene_color_view: &wgpu::TextureView,
        scene_depth_view: &wgpu::TextureView,
        frame: &ViewportRenderFrame,
    ) -> WgpuBufferUploadBatch {
        let plan: ViewportSceneClearPlan =
            frame.camera_stack_attachment_policy().scene_clear_plan();
        if !plan.has_clear() {
            return WgpuBufferUploadBatch::new();
        }
        let color = plan
            .scene_color()
            .map(|clear| resolve_scene_clear_color(clear, frame));
        self.record(
            encoder,
            scene_color_view,
            scene_depth_view,
            frame.render_region(),
            color,
            plan.scene_depth(),
        )
    }

    fn record(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        scene_color_view: &wgpu::TextureView,
        scene_depth_view: &wgpu::TextureView,
        render_region: ViewportRenderRegion,
        color: Option<Vec4>,
        depth: bool,
    ) -> WgpuBufferUploadBatch {
        if render_region.is_empty() || (color.is_none() && !depth) {
            return WgpuBufferUploadBatch::new();
        }
        let mut uploads = WgpuBufferUploadBatch::new();
        if let Some(color) = color {
            uploads.push(WgpuBufferUpload::from_bytes(
                self.color_buffer.clone(),
                0,
                bytemuck::bytes_of(&SceneRegionClearColorUniform::new(color)),
            ));
        }
        match (color.is_some(), depth) {
            (true, true) => {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("SceneRegionClearColorDepthPass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: scene_color_view,
                        resolve_target: None,
                        depth_slice: None,
                        ops: load_store_color_ops(),
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: scene_depth_view,
                        depth_ops: Some(load_store_depth_ops()),
                        stencil_ops: None,
                    }),
                    occlusion_query_set: None,
                    timestamp_writes: None,
                    multiview_mask: None,
                });
                apply_color_depth_clear_state(render_region, &mut pass);
                pass.set_pipeline(&self.color_depth_pipeline);
                pass.set_bind_group(0, &self.color_bind_group, &[]);
                pass.draw(0..3, 0..1);
            }
            (true, false) => {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("SceneRegionClearColorPass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: scene_color_view,
                        resolve_target: None,
                        depth_slice: None,
                        ops: load_store_color_ops(),
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                    multiview_mask: None,
                });
                apply_color_depth_clear_state(render_region, &mut pass);
                pass.set_pipeline(&self.color_pipeline);
                pass.set_bind_group(0, &self.color_bind_group, &[]);
                pass.draw(0..3, 0..1);
            }
            (false, true) => {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("SceneRegionClearDepthPass"),
                    color_attachments: &[],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: scene_depth_view,
                        depth_ops: Some(load_store_depth_ops()),
                        stencil_ops: None,
                    }),
                    occlusion_query_set: None,
                    timestamp_writes: None,
                    multiview_mask: None,
                });
                apply_color_depth_clear_state(render_region, &mut pass);
                pass.set_pipeline(&self.depth_pipeline);
                pass.draw(0..3, 0..1);
            }
            (false, false) => {}
        }
        uploads
    }
}

fn resolve_scene_clear_color(clear: ViewportSceneColorClear, frame: &ViewportRenderFrame) -> Vec4 {
    clear.resolve(frame.preview().clear_color)
}

fn create_color_pipeline(
    device: &wgpu::Device,
    shader: &wgpu::ShaderModule,
    layout: &wgpu::PipelineLayout,
    color_format: wgpu::TextureFormat,
    depth_format: Option<wgpu::TextureFormat>,
    label: &'static str,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(layout),
        vertex: vertex_state(shader),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: depth_format.map(depth_stencil_state),
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some("fs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: color_format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    })
}

fn create_depth_pipeline(
    device: &wgpu::Device,
    shader: &wgpu::ShaderModule,
    layout: &wgpu::PipelineLayout,
    depth_format: wgpu::TextureFormat,
    label: &'static str,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(layout),
        vertex: vertex_state(shader),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: Some(depth_stencil_state(depth_format)),
        multisample: wgpu::MultisampleState::default(),
        fragment: None,
        multiview_mask: None,
        cache: None,
    })
}

fn vertex_state(shader: &wgpu::ShaderModule) -> wgpu::VertexState<'_> {
    wgpu::VertexState {
        module: shader,
        entry_point: Some("vs_main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        buffers: &[],
    }
}

fn depth_stencil_state(format: wgpu::TextureFormat) -> wgpu::DepthStencilState {
    wgpu::DepthStencilState {
        format,
        depth_write_enabled: Some(true),
        depth_compare: Some(wgpu::CompareFunction::Always),
        stencil: wgpu::StencilState::default(),
        bias: wgpu::DepthBiasState::default(),
    }
}

fn load_store_color_ops() -> wgpu::Operations<wgpu::Color> {
    wgpu::Operations {
        load: wgpu::LoadOp::Load,
        store: wgpu::StoreOp::Store,
    }
}

fn load_store_depth_ops() -> wgpu::Operations<f32> {
    wgpu::Operations {
        load: wgpu::LoadOp::Load,
        store: wgpu::StoreOp::Store,
    }
}

fn apply_color_depth_clear_state(
    render_region: ViewportRenderRegion,
    pass: &mut wgpu::RenderPass<'_>,
) {
    render_region.apply_local_to_render_pass(pass);
}

#[cfg(test)]
mod tests {
    use crate::core::math::UVec2;
    use crate::graphics::backend::{OffscreenTarget, RenderBackend};
    use crate::graphics::scene::scene_renderer::core::DEPTH_FORMAT;
    use crate::graphics::scene::scene_renderer::core::SCENE_COLOR_HDR_FORMAT;

    use super::*;

    #[test]
    fn scene_region_clear_resources_build_for_offscreen_backend() {
        let backend = RenderBackend::new_offscreen().unwrap();
        let target = OffscreenTarget::new(&backend.device, UVec2::new(16, 16));
        let resources =
            SceneRegionClearResources::new(&backend.device, SCENE_COLOR_HDR_FORMAT, DEPTH_FORMAT);
        let mut encoder = backend
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("zircon-scene-region-clear-test-encoder"),
            });

        let color_uploads = resources.record(
            &mut encoder,
            &target.scene_color_view,
            &target.depth_view,
            ViewportRenderRegion::full_target(target.size),
            Some(Vec4::ONE),
            true,
        );
        assert!(!color_uploads.is_empty());

        let depth_only_uploads = resources.record(
            &mut encoder,
            &target.scene_color_view,
            &target.depth_view,
            ViewportRenderRegion::full_target(target.size),
            None,
            true,
        );
        assert!(depth_only_uploads.is_empty());

        let _upload_submission = backend
            .enqueue_copy_buffer_upload_batch(color_uploads)
            .unwrap();
        backend.queue.submit([encoder.finish()]);
    }

    #[test]
    fn scene_region_clear_defers_color_upload_to_frame_transaction() {
        let source = include_str!("scene_region_clear_resources.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();
        let stage_source = include_str!(
            "../core/scene_renderer_core_render_compiled_scene/render/execute_compiled_scene_graph_stages.rs"
        );
        let frame_source =
            include_str!("../core/scene_renderer_core_render_compiled_scene/render/render.rs");

        assert!(production.contains("WgpuBufferUpload::from_bytes("));
        assert!(!production.contains("queue.write_buffer("));
        assert!(!production.contains("queue: &wgpu::Queue"));

        let clear_record = stage_source
            .find("let mut scene_clear_uploads = scene_clear.record_frame_clear(")
            .expect("scene clear must prepare its color upload while recording the clear draw");
        let graph_append = stage_source
            .find("graph_execution.append_buffer_uploads(&mut scene_clear_uploads)")
            .expect("scene clear upload must join graph-owned pending uploads");
        assert!(clear_record < graph_append);

        let graph_success = frame_source
            .find("let mut graph_buffer_uploads = graph_execution.take_buffer_uploads()")
            .expect("frame owner must retain graph uploads only after graph success");
        let upload_accept = frame_source
            .find(".enqueue_copy_resource_upload_batch(")
            .expect("frame owner must accept one merged buffer upload batch");
        assert!(graph_success < upload_accept);
    }
}
