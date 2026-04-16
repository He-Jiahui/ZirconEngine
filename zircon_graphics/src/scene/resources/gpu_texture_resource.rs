use zircon_resource::ResourceId;

pub(crate) struct GpuTextureResource {
    #[allow(dead_code)]
    pub(crate) id: Option<ResourceId>,
    #[allow(dead_code)]
    pub(super) texture: wgpu::Texture,
    #[allow(dead_code)]
    pub(super) view: wgpu::TextureView,
    #[allow(dead_code)]
    pub(super) sampler: wgpu::Sampler,
    pub(crate) bind_group: wgpu::BindGroup,
}
