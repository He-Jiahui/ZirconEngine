use crate::scene::selection::SelectionMutation;
use zircon_runtime_interface::math::Vec2;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum SceneModeInputEffect {
    PointerMoved(Vec2),
    PrimaryPressed {
        position: Vec2,
        allow_handle_drag: bool,
        selection_mutation: SelectionMutation,
    },
    PrimaryReleased,
}
