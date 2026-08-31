use crate::scene::selection::SelectionMutation;
use zircon_runtime_interface::math::Vec2;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum SceneModeInputEffect {
    PointerMoved(Vec2),
    SelectionPrimaryPressed {
        position: Vec2,
        selection_mutation: SelectionMutation,
    },
    TransformPrimaryPressed {
        position: Vec2,
        selection_mutation: SelectionMutation,
    },
    PrimaryReleased,
}
