use std::sync::Arc;

use super::viewport_icon_sprite::ViewportIconSprite;
use zr_rhi_wgpu::WgpuTextureUpload;

#[derive(Clone)]
pub(super) enum IconEntry {
    Unloaded,
    Missing,
    Pending {
        sprite: Arc<ViewportIconSprite>,
        upload: WgpuTextureUpload,
    },
    Ready(Arc<ViewportIconSprite>),
}
