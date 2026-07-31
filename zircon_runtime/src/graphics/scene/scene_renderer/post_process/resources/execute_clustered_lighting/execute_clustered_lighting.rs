use crate::core::framework::render::RenderDirectionalLightSnapshot;
use crate::core::math::UVec2;
use bytemuck::Zeroable;

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
        lights: &[RenderDirectionalLightSnapshot],
        enabled: bool,
    ) {
        if !enabled {
            encoder.clear_buffer(cluster_buffer, 0, None);
            return;
        }

        let mut gpu_lights = [ClusteredDirectionalLight::zeroed(); MAX_DIRECTIONAL_LIGHTS];
        let directional_light_count = lights.len().min(MAX_DIRECTIONAL_LIGHTS);
        for (slot, light) in lights.iter().take(directional_light_count).enumerate() {
            gpu_lights[slot] = ClusteredDirectionalLight {
                direction: [light.direction.x, light.direction.y, light.direction.z, 0.0],
                color_intensity: [light.color.x, light.color.y, light.color.z, light.intensity],
            };
        }
        if directional_light_count > 0 {
            queue.write_buffer(
                &self.light_buffer,
                0,
                bytemuck::cast_slice(&gpu_lights[..directional_light_count]),
            );
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

#[cfg(test)]
mod tests {
    #[test]
    fn clustered_lighting_avoids_cpu_clear_and_inactive_light_uploads() {
        let source = include_str!("execute_clustered_lighting.rs");
        let cpu_clear = ["vec![0_u8;", " cluster_buffer_bytes]"].concat();
        let legacy_size_argument = ["cluster_buffer_", "bytes: usize"].concat();
        let gpu_clear = ["encoder.clear_", "buffer(cluster_buffer, 0, None)"].concat();
        let active_prefix = ["&gpu_lights[..", "directional_light_count]"].concat();

        assert!(!source.contains(&cpu_clear));
        assert!(!source.contains(&legacy_size_argument));
        assert!(source.contains(&gpu_clear));
        assert!(source.contains(&active_prefix));
    }
}
