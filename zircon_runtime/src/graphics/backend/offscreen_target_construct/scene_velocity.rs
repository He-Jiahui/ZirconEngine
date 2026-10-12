use super::super::offscreen_target::{OffscreenTarget, OffscreenTargetSceneVelocity};
use super::create_texture_bundle::create_texture_bundle;

impl OffscreenTarget {
    pub(crate) fn ensure_scene_velocity(&mut self, device: &wgpu::Device) {
        if self.scene_velocity.is_some() {
            return;
        }
        let velocity = create_texture_bundle(
            device,
            "zircon-offscreen-scene-velocity",
            self.render_size,
            wgpu::TextureFormat::Rg16Float,
            wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
        );
        self.scene_velocity = Some(OffscreenTargetSceneVelocity {
            texture: velocity.texture,
            view: velocity.view,
            identity: crate::graphics::resource_identity::SampledTextureIdentity::new(),
        });
    }

    pub(crate) fn scene_velocity(
        &self,
    ) -> Option<(
        &wgpu::Texture,
        &wgpu::TextureView,
        crate::graphics::resource_identity::SampledTextureIdentity,
    )> {
        self.scene_velocity
            .as_ref()
            .map(|velocity| (&velocity.texture, &velocity.view, velocity.identity))
    }
}
