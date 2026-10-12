use crate::core::framework::render::ViewportIconId;

pub(super) fn icon_slot(id: ViewportIconId) -> usize {
    match id {
        ViewportIconId::Camera => 0,
        ViewportIconId::DirectionalLight => 1,
    }
}
