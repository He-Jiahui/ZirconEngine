use zircon_resource::ResourceId;

pub(crate) struct GpuTextureResource {
    #[allow(dead_code)]
    pub(crate) id: Option<ResourceId>,
    #[allow(dead_code)]
    pub(in crate::scene::resources) texture: wgpu::Texture,
    #[allow(dead_code)]
    pub(in crate::scene::resources) view: wgpu::TextureView,
    #[allow(dead_code)]
    pub(in crate::scene::resources) sampler: wgpu::Sampler,
    pub(crate) bind_group: wgpu::BindGroup,
}
