use crate::core::framework::render::OverlayBillboardIcon;

pub(in crate::graphics::scene::scene_renderer::primitives) fn icon_world_size(
    icon: &OverlayBillboardIcon,
) -> f32 {
    (icon.size * 0.0035).max(0.04)
}
