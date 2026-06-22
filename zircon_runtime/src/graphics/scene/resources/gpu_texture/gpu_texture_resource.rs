use crate::core::framework::render::RenderImageDescriptor;
use crate::core::resource::ResourceId;

pub(crate) struct GpuTextureResource {
    pub(crate) id: Option<ResourceId>,
    pub(crate) descriptor: RenderImageDescriptor,
    pub(in crate::graphics::scene::resources) texture: wgpu::Texture,
    pub(in crate::graphics::scene::resources) view: wgpu::TextureView,
    pub(in crate::graphics::scene::resources) sampler: wgpu::Sampler,
    pub(crate) bind_group: wgpu::BindGroup,
}

impl GpuTextureResource {
    pub(crate) const RETAINED_TEXTURE_BINDING_OWNER_COUNT: usize = 4;

    pub(crate) fn retained_texture_binding_owner_count(&self) -> usize {
        let _retained_texture_binding_owners = (&self.id, &self.texture, &self.view, &self.sampler);
        Self::RETAINED_TEXTURE_BINDING_OWNER_COUNT
    }

    pub(crate) fn view(&self) -> &wgpu::TextureView {
        debug_assert_eq!(
            self.retained_texture_binding_owner_count(),
            Self::RETAINED_TEXTURE_BINDING_OWNER_COUNT,
            "GpuTextureResource must retain identity, texture, view, and sampler while exposing bindings",
        );
        &self.view
    }

    pub(crate) fn sampler(&self) -> &wgpu::Sampler {
        debug_assert_eq!(
            self.retained_texture_binding_owner_count(),
            Self::RETAINED_TEXTURE_BINDING_OWNER_COUNT,
            "GpuTextureResource must retain identity, texture, view, and sampler while exposing bindings",
        );
        &self.sampler
    }
}
