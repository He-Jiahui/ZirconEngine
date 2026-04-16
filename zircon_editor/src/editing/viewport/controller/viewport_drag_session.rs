use zircon_math::Vec2;

use crate::editing::viewport::handles::HandleDragSession;

#[derive(Clone, Debug)]
pub(crate) enum ViewportDragSession {
    PrimaryNavigation {
        start: Vec2,
        active: bool,
        target: Option<u64>,
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
