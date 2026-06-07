use super::super::geometry_pipeline::create_geometry_pipeline;
use super::super::lighting_bind_group_layout::create_lighting_bind_group_layout;
use super::super::lighting_pipeline::create_lighting_pipeline;
use super::shadow_receiver_uniform::DeferredShadowReceiverUniform;
use super::DeferredSceneResources;

impl DeferredSceneResources {
    pub(crate) fn new(
        device: &wgpu::Device,
        scene_layout: &wgpu::BindGroupLayout,
        model_layout: &wgpu::BindGroupLayout,
        texture_layout: &wgpu::BindGroupLayout,
        material_layout: &wgpu::BindGroupLayout,
        target_format: wgpu::TextureFormat,
    ) -> Self {
        let geometry_pipeline = create_geometry_pipeline(
            device,
            scene_layout,
            model_layout,
            texture_layout,
            material_layout,
        );
        let lighting_bind_group_layout = create_lighting_bind_group_layout(device);
        let lighting_pipeline = create_lighting_pipeline(
            device,
            scene_layout,
            &lighting_bind_group_layout,
            target_format,
        );
        let shadow_receiver_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-deferred-shadow-receiver-uniform"),
            size: std::mem::size_of::<DeferredShadowReceiverUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let shadow_compare_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("zircon-deferred-shadow-compare-sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });

        Self {
            geometry_pipeline,
            lighting_bind_group_layout,
            lighting_pipeline,
            shadow_receiver_uniform_buffer,
            shadow_compare_sampler,
        }
    }
}
