use crate::core::math::UVec2;
use crate::graphics::scene::scene_renderer::post_process::hzb_build_dispatch_groups;

use super::super::super::params::hzb_params::HzbParams;
use super::super::super::scene_post_process_resources::ScenePostProcessResources;

impl ScenePostProcessResources {
    pub(crate) fn execute_hzb_build_mip(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        scene_depth_view: &wgpu::TextureView,
        source_hzb_view: Option<&wgpu::TextureView>,
        target_hzb_view: &wgpu::TextureView,
        target_size: UVec2,
        target_mip_level: u32,
    ) {
        let params = HzbParams {
            target_size: [target_size.x.max(1), target_size.y.max(1)],
            target_mip_level,
            _pad0: 0,
        };
        queue.write_buffer(&self.hzb_params_buffer, 0, bytemuck::bytes_of(&params));

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-hzb-build-bind-group"),
            layout: &self.hzb_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(scene_depth_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        source_hzb_view.unwrap_or(&self.hzb_source_texture_view),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.hzb_params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(target_hzb_view),
                },
            ],
        });

        let dispatch_groups = hzb_build_dispatch_groups(target_size);
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("HzbBuildPass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.hzb_pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(dispatch_groups[0], dispatch_groups[1], dispatch_groups[2]);
    }
}
