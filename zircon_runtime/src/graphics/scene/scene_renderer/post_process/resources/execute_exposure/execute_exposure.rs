use crate::core::framework::render::RenderExposureSettings;
use crate::core::math::UVec2;
use crate::graphics::scene::scene_renderer::post_process::{
    exposure_histogram_dispatch_groups, exposure_resolve_dispatch_groups,
};

use super::super::super::params::exposure_params::ExposureParams;
use super::super::super::scene_post_process_resources::ScenePostProcessResources;

const EXPOSURE_ADAPTATION_DELTA_SECONDS: f32 = 1.0 / 60.0;

impl ScenePostProcessResources {
    pub(crate) fn execute_exposure_histogram(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        viewport_size: UVec2,
        scene_color_view: &wgpu::TextureView,
        histogram_buffer: &wgpu::Buffer,
        settings: RenderExposureSettings,
    ) {
        let params =
            ExposureParams::new(viewport_size, settings, EXPOSURE_ADAPTATION_DELTA_SECONDS);
        queue.write_buffer(&self.exposure_params_buffer, 0, bytemuck::bytes_of(&params));
        encoder.clear_buffer(histogram_buffer, 0, None);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-exposure-histogram-bind-group"),
            layout: &self.exposure_histogram_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(scene_color_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.exposure_params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: histogram_buffer.as_entire_binding(),
                },
            ],
        });

        let dispatch_groups = exposure_histogram_dispatch_groups(viewport_size);
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("ExposureHistogramPass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.exposure_histogram_pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(dispatch_groups[0], dispatch_groups[1], dispatch_groups[2]);
    }

    pub(crate) fn execute_exposure_resolve(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        viewport_size: UVec2,
        histogram_buffer: &wgpu::Buffer,
        previous_exposure_buffer: &wgpu::Buffer,
        current_exposure_buffer: &wgpu::Buffer,
        settings: RenderExposureSettings,
    ) {
        let params =
            ExposureParams::new(viewport_size, settings, EXPOSURE_ADAPTATION_DELTA_SECONDS);
        queue.write_buffer(&self.exposure_params_buffer, 0, bytemuck::bytes_of(&params));

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-exposure-resolve-bind-group"),
            layout: &self.exposure_resolve_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.exposure_params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: histogram_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: previous_exposure_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: current_exposure_buffer.as_entire_binding(),
                },
            ],
        });

        let dispatch_groups = exposure_resolve_dispatch_groups();
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("ExposureResolvePass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.exposure_resolve_pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(dispatch_groups[0], dispatch_groups[1], dispatch_groups[2]);
    }
}
