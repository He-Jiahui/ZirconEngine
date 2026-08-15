use crate::graphics::resource_identity::SampledTextureIdentity;

use super::RenderGraphExecutionResources;

impl RenderGraphExecutionResources {
    pub(in crate::graphics::scene::scene_renderer) fn import_texture_view_with_identity(
        &mut self,
        name: impl Into<String>,
        view: wgpu::TextureView,
        identity: SampledTextureIdentity,
    ) -> Option<wgpu::TextureView> {
        let name = name.into();
        let previous = self.import_texture_view(name.clone(), view);
        self.set_texture_identity(name, identity);
        previous
    }

    pub(in crate::graphics::scene::scene_renderer) fn import_borrowed_texture_view_with_identity(
        &mut self,
        name: impl Into<String>,
        view: &wgpu::TextureView,
        identity: SampledTextureIdentity,
    ) -> Option<wgpu::TextureView> {
        self.import_texture_view_with_identity(name, view.clone(), identity)
    }

    pub(in crate::graphics::scene::scene_renderer) fn texture_identity(
        &self,
        name: &str,
    ) -> Option<SampledTextureIdentity> {
        self.sampled_texture_identities.get(name).copied()
    }

    pub(super) fn set_texture_identity(
        &mut self,
        name: impl Into<String>,
        identity: SampledTextureIdentity,
    ) {
        self.sampled_texture_identities
            .insert(name.into(), identity);
    }
}
