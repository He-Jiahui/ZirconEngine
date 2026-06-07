use crate::core::framework::render::{RenderDepthOfFieldSettings, ViewportCameraSnapshot};
use crate::core::math::UVec2;

use super::super::super::clear_render_target::clear_render_target;
use super::super::super::depth_of_field_prepare_params::{
    depth_of_field_prepare_enabled, DepthOfFieldPrepareParams,
};
use super::super::super::resources::depth_sampling_mode::PostProcessDepthSamplingMode;
use super::super::super::scene_post_process_resources::ScenePostProcessResources;

impl ScenePostProcessResources {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn execute_depth_of_field_prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        viewport_size: UVec2,
        scene_color_view: &wgpu::TextureView,
        scene_depth_view: &wgpu::TextureView,
        coc_view: &wgpu::TextureView,
        bokeh_view: &wgpu::TextureView,
        settings: RenderDepthOfFieldSettings,
        camera: &ViewportCameraSnapshot,
    ) {
        if !depth_of_field_prepare_enabled(settings) {
            clear_render_target(
                encoder,
                "ClearDepthOfFieldCocPass",
                coc_view,
                wgpu::Color::BLACK,
            );
            clear_render_target(
                encoder,
                "ClearDepthOfFieldBokehPass",
                bokeh_view,
                wgpu::Color::BLACK,
            );
            return;
        }

        let params = DepthOfFieldPrepareParams::from_camera(viewport_size, camera, settings);
        queue.write_buffer(
            &self.depth_of_field_prepare_params_buffer,
            0,
            bytemuck::bytes_of(&params),
        );
        let scene_depth_binding_view = match self.depth_sampling_mode {
            PostProcessDepthSamplingMode::RawDepthTexture => scene_depth_view,
            PostProcessDepthSamplingMode::ViewportDepthFallback => &self.black_texture_view,
        };
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-depth-of-field-prepare-bind-group"),
            layout: &self.depth_of_field_prepare_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(scene_depth_binding_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self
                        .depth_of_field_prepare_params_buffer
                        .as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(scene_color_view),
                },
            ],
        });

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("DepthOfFieldPreparePass"),
            color_attachments: &[
                Some(wgpu::RenderPassColorAttachment {
                    view: coc_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                }),
                Some(wgpu::RenderPassColorAttachment {
                    view: bokeh_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                }),
            ],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.depth_of_field_prepare_pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}
