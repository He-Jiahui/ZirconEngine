use crate::core::framework::render::RenderExposureSettings;
use crate::core::math::UVec2;
use crate::graphics::scene::scene_renderer::post_process::{
    exposure_histogram_dispatch_groups, exposure_resolve_dispatch_groups,
};
use zr_rhi_wgpu::{WgpuBufferUpload, WgpuBufferUploadBatch};

use super::super::super::params::exposure_params::ExposureParams;
use super::super::super::scene_post_process_resources::ScenePostProcessResources;

impl ScenePostProcessResources {
    pub(crate) fn prepare_exposure_params_upload(
        &self,
        viewport_size: UVec2,
        settings: RenderExposureSettings,
        raw_real_delta_seconds: f32,
        frame_uploads: &mut WgpuBufferUploadBatch,
    ) {
        let params = ExposureParams::new(viewport_size, settings, raw_real_delta_seconds);
        frame_uploads.push(WgpuBufferUpload::from_bytes(
            self.exposure_params_buffer.clone(),
            0,
            bytemuck::bytes_of(&params),
        ));
    }

    pub(crate) fn execute_exposure_histogram(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        viewport_size: UVec2,
        scene_color_view: &wgpu::TextureView,
        histogram_buffer: wgpu::BufferBinding<'_>,
    ) {
        encoder.clear_buffer(
            histogram_buffer.buffer,
            histogram_buffer.offset,
            histogram_buffer.size.map(std::num::NonZeroU64::get),
        );

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
                    resource: wgpu::BindingResource::Buffer(histogram_buffer),
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
        encoder: &mut wgpu::CommandEncoder,
        histogram_buffer: wgpu::BufferBinding<'_>,
        previous_exposure_buffer: wgpu::BufferBinding<'_>,
        current_exposure_buffer: wgpu::BufferBinding<'_>,
    ) {
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
                    resource: wgpu::BindingResource::Buffer(histogram_buffer),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(previous_exposure_buffer),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Buffer(current_exposure_buffer),
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

#[cfg(test)]
mod tests {
    #[test]
    fn exposure_params_have_one_frame_preparation_owner() {
        let source = include_str!("execute_exposure.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("exposure production source");

        assert_eq!(production.matches("ExposureParams::new(").count(), 1);
        assert_eq!(
            production.matches("prepare_exposure_params_upload").count(),
            1
        );
        assert!(!production.contains("queue.write_buffer"));
        assert!(production.contains("WgpuBufferUpload::from_bytes("));
        assert!(!production.contains("EXPOSURE_ADAPTATION_DELTA_SECONDS"));
        assert!(production.contains("raw_real_delta_seconds"));
    }
}
