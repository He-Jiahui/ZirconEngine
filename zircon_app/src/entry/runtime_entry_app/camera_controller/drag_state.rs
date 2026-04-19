use zircon_runtime::core::math::Vec2;

#[derive(Clone, Copy, Debug)]
pub(super) enum DragState {
    Orbit { last: Vec2 },
    Pan { last: Vec2 },
}
