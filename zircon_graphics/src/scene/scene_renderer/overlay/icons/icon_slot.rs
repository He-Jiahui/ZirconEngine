use zircon_scene::ViewportIconId;

pub(super) fn icon_slot(id: ViewportIconId) -> usize {
    match id {
        ViewportIconId::Camera => 0,
        ViewportIconId::DirectionalLight => 1,
    }
}
