mod bind_group_layout;
mod node_and_cluster_cull_instance_work_item_pipeline;
mod params_buffer;
mod uploader_pipeline;

use bind_group_layout::create_uploader_bind_group_layout;
use node_and_cluster_cull_instance_work_item_pipeline::create_node_and_cluster_cull_instance_work_item_pipeline;
use params_buffer::create_uploader_params_buffer;
use uploader_pipeline::create_uploader_pipeline;

use super::VirtualGeometryGpuResources;

impl VirtualGeometryGpuResources {
    pub(in crate::graphics::scene::scene_renderer) fn new(device: &wgpu::Device) -> Self {
        let bind_group_layout = create_uploader_bind_group_layout(device);
        let pipeline = create_uploader_pipeline(device, &bind_group_layout);
        let params_buffer = create_uploader_params_buffer(device);
        let (
            node_and_cluster_cull_instance_work_item_bind_group_layout,
            node_and_cluster_cull_instance_work_item_pipeline,
        ) = create_node_and_cluster_cull_instance_work_item_pipeline(device);

        Self {
            bind_group_layout,
            pipeline,
            params_buffer,
            node_and_cluster_cull_instance_work_item_bind_group_layout,
            node_and_cluster_cull_instance_work_item_pipeline,
        }
    }
}
