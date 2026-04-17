use zircon_math::Vec2;

#[derive(Clone, Copy, Debug)]
pub(super) enum DragState {
    Orbit { last: Vec2 },
    Pan { last: Vec2 },
}
