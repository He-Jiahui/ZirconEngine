use bytemuck::Zeroable;
use crate::core::framework::render::RenderDirectionalLightSnapshot;
use crate::core::math::UVec2;

use super::super::super::cluster_params::ClusterParams;
use super::super::super::clustered_directional_light::ClusteredDirectionalLight;
use super::super::super::constants::{
    CLUSTER_TILE_SIZE, CLUSTER_WORKGROUP_SIZE, MAX_DIRECTIONAL_LIGHTS,
};
use super::super::super::scene_post_process_resources::ScenePostProcessResources;

impl ScenePostProcessResources {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn execute_clustered_lighting(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        viewport_size: UVec2,
        cluster_dimensions: UVec2,
        cluster_buffer: &wgpu::Buffer,
        cluster_buffer_bytes: usize,
        lights: &[RenderDirectionalLightSnapshot],
        enabled: bool,
    ) {
        if !enabled {
            queue.write_buffer(cluster_buffer, 0, &vec![0_u8; cluster_buffer_bytes]);
            return;
        }

        let mut gpu_lights = [ClusteredDirectionalLight::zeroed(); MAX_DIRECTIONAL_LIGHTS];
        for (slot, light) in lights.iter().take(MAX_DIRECTIONAL_LIGHTS).enumerate() {
            gpu_lights[slot] = ClusteredDirectionalLight {
                direction: [light.direction.x, light.direction.y, light.direction.z, 0.0],
                color_intensity: [light.color.x, light.color.y, light.color.z, light.intensity],
            };
        }
        queue.write_buffer(&self.light_buffer, 0, bytemuck::cast_slice(&gpu_lights));

        let params = ClusterParams {
            viewport_and_clusters: [
                viewport_size.x.max(1),
                viewport_size.y.max(1),
                cluster_dimensions.x.max(1),
                cluster_dimensions.y.max(1),
            ],
            counts: [
                lights.len().min(MAX_DIRECTIONAL_LIGHTS) as u32,
                CLUSTER_TILE_SIZE,
                0,
                0,
            ],
            strengths: [0.42, 0.18, 0.0, 0.0],
        };
        queue.write_buffer(&self.cluster_params_buffer, 0, bytemuck::bytes_of(&params));

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
                    resource: cluster_buffer.as_entire_binding(),
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
    }
}
