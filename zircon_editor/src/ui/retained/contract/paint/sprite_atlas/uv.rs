use zircon_runtime::asset::SpriteAtlasUvRect;

use super::super::super::paint_frame::HostPaintImageUvRect;

/// 图集 UV 由资源矩形投影给宿主命令，显示框缩放不应反向修改源像素坐标。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn host_uv_rect(
    uv: SpriteAtlasUvRect,
) -> HostPaintImageUvRect {
    HostPaintImageUvRect {
        min: uv.min,
        max: uv.max,
    }
}
