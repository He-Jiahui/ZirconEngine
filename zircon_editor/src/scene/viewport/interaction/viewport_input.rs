use zircon_runtime::core::math::{UVec2, Vec2};

#[derive(Clone, Debug)]
pub enum ViewportInput {
    PointerMoved(Vec2),
    LeftPressed(Vec2),
    LeftReleased,
    RightPressed(Vec2),
    RightReleased,
    MiddlePressed(Vec2),
    MiddleReleased,
    Scrolled(f32),
    Resized(UVec2),
}
