use zircon_runtime_interface::math::Vec2;

use crate::scene::selection::SelectionMutation;
use crate::scene::viewport::handles::HandleDragSession;

#[derive(Clone, Debug)]
pub(crate) enum ViewportDragSession {
    PrimarySelection {
        start: Vec2,
        current: Vec2,
        active: bool,
        target: Option<u64>,
        mutation: SelectionMutation,
    },
    Orbit {
        last: Vec2,
    },
    Pan {
        last: Vec2,
    },
    Handle {
        session: HandleDragSession,
    },
}
