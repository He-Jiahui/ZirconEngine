use zircon_math::UVec2;

use super::super::super::clear_render_target::clear_render_target;
use super::super::super::constants::SSAO_WORKGROUP_SIZE;
use super::super::super::scene_post_process_resources::ScenePostProcessResources;
use super::super::super::ssao_params::SsaoParams;

impl ScenePostProcessResources {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn execute_ssao(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        viewport_size: UVec2,
        depth_view: &wgpu::TextureView,
        normal_view: &wgpu::TextureView,
        previous_ao_view: Option<&wgpu::TextureView>,
        ao_view: &wgpu::TextureView,
        enabled: bool,
        history_available: bool,
    ) {
        if !enabled {
            clear_render_target(
                encoder,
                "ClearAmbientOcclusion",
                ao_view,
                wgpu::Color::WHITE,
            );
            return;
        }

        let params = SsaoParams {
            viewport_and_flags: [
                viewport_size.x.max(1),
                viewport_size.y.max(1),
                u32::from(history_available),
                0,
            ],
            tuning: [4.6, 0.0015, 0.18, 0.88],
        };
        queue.write_buffer(&self.ssao_params_buffer, 0, bytemuck::bytes_of(&params));

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-ssao-bind-group"),
            layout: &self.ssao_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(depth_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(normal_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(
                        previous_ao_view.unwrap_or(&self.white_texture_view),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: self.ssao_params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(ao_view),
                },
            ],
        });

        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("SsaoEvaluatePass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.ssao_pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(
            viewport_size.x.max(1).div_ceil(SSAO_WORKGROUP_SIZE),
            viewport_size.y.max(1).div_ceil(SSAO_WORKGROUP_SIZE),
            1,
        );
    }
}
