use crate::core::math::Vec2;

use super::{PointerButton, PointerId, PointerLocation};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PointerScrollUnit {
    Line,
    Pixel,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PointerAction {
    Press(PointerButton),
    Release(PointerButton),
    Move {
        delta: Vec2,
    },
    Scroll {
        unit: PointerScrollUnit,
        delta: Vec2,
    },
    Cancel,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointerInput {
    pub location: PointerLocation,
    pub action: PointerAction,
}

impl PointerInput {
    pub const fn new(location: PointerLocation, action: PointerAction) -> Self {
        Self { location, action }
    }

    pub const fn pointer(self) -> PointerId {
        self.location.pointer
    }
}
