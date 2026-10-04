use std::sync::Arc;

/// 让纹理、视图和绘制绑定共享同一存活期；准备结果克隆绑定时不转移缓存所有权。
pub(super) struct ViewportIconSprite {
    pub(super) _texture: wgpu::Texture,
    pub(super) _view: wgpu::TextureView,
    pub(super) bind_group: Arc<wgpu::BindGroup>,
}
