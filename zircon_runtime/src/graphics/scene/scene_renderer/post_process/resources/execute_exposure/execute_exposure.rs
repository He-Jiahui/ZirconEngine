use crate::core::framework::render::RenderExposureSettings;
use crate::core::math::UVec2;
use crate::graphics::scene::scene_renderer::post_process::{
    exposure_histogram_dispatch_groups, exposure_resolve_dispatch_groups,
};
use zr_rhi_wgpu::{WgpuBufferUpload, WgpuBufferUploadBatch};

use super::super::super::params::exposure_params::ExposureParams;
use super::super::super::scene_post_process_resources::ScenePostProcessResources;

impl ScenePostProcessResources {
    /// 帧基础准备阶段为 histogram 与 resolve 提供同一份曝光参数，使用权威真实帧间隔。
    /// 上传批次须在这两个计算节点提交前执行；两节点之间无需重复更新该持久 uniform。
    pub(crate) fn prepare_exposure_params_upload(
        &self,
        viewport_size: UVec2,
        settings: RenderExposureSettings,
        raw_real_delta_seconds: f32,
        frame_uploads: &mut WgpuBufferUploadBatch,
    ) {
        // TODO: [CR-SCENE-POST-0007] 核对动态分辨率或局部视口下，帧基础传入的 target.size 是否与 histogram 的 scene_linear_size 一致；此处尺寸也决定统计像素总数。
        let params = ExposureParams::new(viewport_size, settings, raw_real_delta_seconds);
        frame_uploads.push(WgpuBufferUpload::from_bytes(
            self.exposure_params_buffer.clone(),
            0,
            bytemuck::bytes_of(&params),
        ));
    }

    /// 从场景线性色输入生成当前帧亮度分布；图节点仅在 Histogram 模式调用。
    /// 调用方提供可清除且可写的统计绑定范围，并保证准备阶段的尺寸覆盖此次采样区域。
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

    /// 结合当前统计和上一帧曝光生成当前曝光，供 LUT 烘焙与组合阶段读取。
    /// Histogram 模式要求统计节点先执行；手动模式可用默认统计，历史和当前绑定须满足图读写约束。
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
#[path = "tests/execute_exposure.rs"]
mod tests;
