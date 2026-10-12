use std::sync::Arc;

use super::viewport_icon_sprite::ViewportIconSprite;
use zr_rhi_wgpu::WgpuTextureUpload;

/// 图标槽的跨帧状态；准备后的绑定可参与本帧绘制，上传债务在提交成功前保留。
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
