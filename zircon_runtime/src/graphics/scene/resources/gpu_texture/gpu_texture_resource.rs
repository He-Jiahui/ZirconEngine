use crate::core::framework::render::RenderImageDescriptor;
use crate::core::resource::ResourceId;

pub(crate) struct GpuTextureResource {
    #[allow(dead_code)]
    pub(crate) id: Option<ResourceId>,
    pub(crate) descriptor: RenderImageDescriptor,
    #[allow(dead_code)]
    pub(in crate::graphics::scene::resources) texture: wgpu::Texture,
    #[allow(dead_code)]
    pub(in crate::graphics::scene::resources) view: wgpu::TextureView,
    #[allow(dead_code)]
    pub(in crate::graphics::scene::resources) sampler: wgpu::Sampler,
    pub(crate) bind_group: wgpu::BindGroup,
}
