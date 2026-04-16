use std::sync::Arc;

pub(super) struct ViewportIconSprite {
    pub(super) _texture: wgpu::Texture,
    pub(super) _view: wgpu::TextureView,
    pub(super) bind_group: Arc<wgpu::BindGroup>,
}
