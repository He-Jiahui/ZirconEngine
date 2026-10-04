use std::sync::Arc;

use crate::core::framework::render::RenderDirectionalLightSnapshot;
use crate::core::math::UVec2;
use bytemuck::Zeroable;
use zr_rhi_wgpu::{WgpuBufferUpload, WgpuBufferUploadBatch};

use super::super::super::cluster_params::ClusterParams;
use super::super::super::clustered_directional_light::ClusteredDirectionalLight;
use super::super::super::constants::{
    CLUSTER_TILE_SIZE, CLUSTER_WORKGROUP_SIZE, MAX_DIRECTIONAL_LIGHTS,
};
use super::super::super::scene_post_process_resources::ScenePostProcessResources;

impl ScenePostProcessResources {
    /// 把当前有效方向光聚合到二维视口网格，并返回提交前应执行的参数/数据上传。
    /// 停用时只清除传入的缓冲绑定范围，调用者须保证该范围覆盖对应网格。
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn execute_clustered_lighting(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        viewport_size: UVec2,
        cluster_dimensions: UVec2,
        cluster_buffer: wgpu::BufferBinding<'_>,
        lights: &[RenderDirectionalLightSnapshot],
        enabled: bool,
    ) -> WgpuBufferUploadBatch {
        if !enabled {
            encoder.clear_buffer(
                cluster_buffer.buffer,
                cluster_buffer.offset,
                cluster_buffer.size.map(std::num::NonZeroU64::get),
            );
            return WgpuBufferUploadBatch::new();
        }

        let mut gpu_lights = [ClusteredDirectionalLight::zeroed(); MAX_DIRECTIONAL_LIGHTS];
        let directional_light_count = lights.len().min(MAX_DIRECTIONAL_LIGHTS);
        for (slot, light) in lights.iter().take(directional_light_count).enumerate() {
            gpu_lights[slot] = ClusteredDirectionalLight {
                direction: [light.direction.x, light.direction.y, light.direction.z, 0.0],
                color_intensity: [light.color.x, light.color.y, light.color.z, light.intensity],
            };
        }
        let params = ClusterParams {
            viewport_and_clusters: [
                viewport_size.x.max(1),
                viewport_size.y.max(1),
                cluster_dimensions.x.max(1),
                cluster_dimensions.y.max(1),
            ],
            counts: [directional_light_count as u32, CLUSTER_TILE_SIZE, 0, 0],
            strengths: [0.42, 0.18, 0.0, 0.0],
        };
        let lights = bytemuck::cast_slice(&gpu_lights[..directional_light_count]);
        let params = bytemuck::bytes_of(&params);
        let mut bytes = Vec::with_capacity(lights.len().saturating_add(params.len()));
        bytes.extend_from_slice(lights);
        let params_start = bytes.len();
        bytes.extend_from_slice(params);
        let payload: Arc<[u8]> = bytes.into();
        let mut uploads = WgpuBufferUploadBatch::new();
        if directional_light_count > 0 {
            if let Some(upload) = WgpuBufferUpload::new(
                self.light_buffer.clone(),
                0,
                payload.clone(),
                0..params_start,
            ) {
                uploads.push(upload);
            }
        }
        if let Some(upload) = WgpuBufferUpload::new(
            self.cluster_params_buffer.clone(),
            0,
            payload,
            params_start..params_start.saturating_add(params.len()),
        ) {
            uploads.push(upload);
        }

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-cluster-bind-group"),
            layout: &self.cluster_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.cluster_params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.light_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(cluster_buffer),
                },
            ],
        });

        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("ClusteredLightCullingPass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.cluster_pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(
            cluster_dimensions.x.max(1).div_ceil(CLUSTER_WORKGROUP_SIZE),
            cluster_dimensions.y.max(1).div_ceil(CLUSTER_WORKGROUP_SIZE),
            1,
        );
        uploads
    }
}

#[cfg(test)]
#[path = "tests/execute_clustered_lighting.rs"]
mod tests;
