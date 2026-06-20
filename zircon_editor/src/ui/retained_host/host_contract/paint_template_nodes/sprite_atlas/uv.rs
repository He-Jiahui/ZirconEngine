use zircon_runtime::asset::SpriteAtlasUvRect;

use super::super::super::paint_frame::HostPaintImageUvRect;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn host_uv_rect(
    uv: SpriteAtlasUvRect,
) -> HostPaintImageUvRect {
    HostPaintImageUvRect {
        min: uv.min,
        max: uv.max,
    }
}
