use zircon_scene::OverlayBillboardIcon;

pub(in crate::scene::scene_renderer::primitives) fn icon_world_size(
    icon: &OverlayBillboardIcon,
) -> f32 {
    (icon.size * 0.0035).max(0.04)
}
