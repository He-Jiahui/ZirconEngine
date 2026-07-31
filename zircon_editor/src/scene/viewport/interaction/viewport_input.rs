use crate::scene::selection::SelectionMutation;
use zircon_runtime_interface::math::{UVec2, Vec2};

#[derive(Clone, Debug)]
pub enum ViewportInput {
    PointerMoved(Vec2),
    LeftPressed {
        position: Vec2,
        selection_mutation: SelectionMutation,
    },
    LeftReleased,
    RightPressed(Vec2),
    RightReleased,
    MiddlePressed(Vec2),
    MiddleReleased,
    Scrolled(f32),
    Resized(UVec2),
}
