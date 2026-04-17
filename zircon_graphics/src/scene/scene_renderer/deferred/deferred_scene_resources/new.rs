use super::super::geometry_pipeline::create_geometry_pipeline;
use super::super::lighting_bind_group_layout::create_lighting_bind_group_layout;
use super::super::lighting_pipeline::create_lighting_pipeline;
use super::DeferredSceneResources;

impl DeferredSceneResources {
    pub(crate) fn new(
        device: &wgpu::Device,
        scene_layout: &wgpu::BindGroupLayout,
        model_layout: &wgpu::BindGroupLayout,
        texture_layout: &wgpu::BindGroupLayout,
        target_format: wgpu::TextureFormat,
    ) -> Self {
        let geometry_pipeline =
            create_geometry_pipeline(device, scene_layout, model_layout, texture_layout);
        let lighting_bind_group_layout = create_lighting_bind_group_layout(device);
        let lighting_pipeline = create_lighting_pipeline(
            device,
            scene_layout,
            &lighting_bind_group_layout,
            target_format,
        );

        Self {
            geometry_pipeline,
            lighting_bind_group_layout,
            lighting_pipeline,
        }
    }
}
