use crate::core::math::Vec2;

use super::{HitData, HitTarget, PointerButton, PointerId, PointerLocation, PointerScrollUnit};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PickingEventLabel {
    Over,
    Enter,
    Move,
    Leave,
    Out,
    Press,
    Release,
    Click,
    DragStart,
    Drag,
    DragEnd,
    DragEnter,
    DragOver,
    DragDrop,
    DragLeave,
    Scroll,
    Cancel,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PickingEventKind {
    Over {
        hit: HitData,
    },
    Enter {
        hit: HitData,
        is_direct: bool,
    },
    Move {
        hit: HitData,
        delta: Vec2,
    },
    Leave {
        hit: HitData,
        was_direct: bool,
    },
    Out {
        hit: HitData,
    },
    Press {
        button: PointerButton,
        hit: HitData,
    },
    Release {
        button: PointerButton,
        hit: HitData,
    },
    Click {
        button: PointerButton,
        hit: HitData,
    },
    DragStart {
        button: PointerButton,
        hit: HitData,
    },
    Drag {
        button: PointerButton,
        distance: Vec2,
        delta: Vec2,
    },
    DragEnd {
        button: PointerButton,
        distance: Vec2,
    },
    DragEnter {
        button: PointerButton,
        dragged: HitTarget,
        hit: HitData,
    },
    DragOver {
        button: PointerButton,
        dragged: HitTarget,
        hit: HitData,
    },
    DragDrop {
        button: PointerButton,
        dropped: HitTarget,
        hit: HitData,
    },
    DragLeave {
        button: PointerButton,
        dragged: HitTarget,
        hit: HitData,
    },
    Scroll {
        unit: PointerScrollUnit,
        delta: Vec2,
        hit: HitData,
    },
    Cancel {
        hit: HitData,
    },
}

impl PickingEventKind {
    pub const fn label(&self) -> PickingEventLabel {
        match self {
            Self::Over { .. } => PickingEventLabel::Over,
            Self::Enter { .. } => PickingEventLabel::Enter,
            Self::Move { .. } => PickingEventLabel::Move,
            Self::Leave { .. } => PickingEventLabel::Leave,
            Self::Out { .. } => PickingEventLabel::Out,
            Self::Press { .. } => PickingEventLabel::Press,
            Self::Release { .. } => PickingEventLabel::Release,
            Self::Click { .. } => PickingEventLabel::Click,
            Self::DragStart { .. } => PickingEventLabel::DragStart,
            Self::Drag { .. } => PickingEventLabel::Drag,
            Self::DragEnd { .. } => PickingEventLabel::DragEnd,
            Self::DragEnter { .. } => PickingEventLabel::DragEnter,
            Self::DragOver { .. } => PickingEventLabel::DragOver,
            Self::DragDrop { .. } => PickingEventLabel::DragDrop,
            Self::DragLeave { .. } => PickingEventLabel::DragLeave,
            Self::Scroll { .. } => PickingEventLabel::Scroll,
            Self::Cancel { .. } => PickingEventLabel::Cancel,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PickingPointerEvent {
    pub pointer: PointerId,
    pub location: PointerLocation,
    pub target: HitTarget,
    pub kind: PickingEventKind,
    pub propagate: bool,
}

impl PickingPointerEvent {
    pub const fn new(
        pointer: PointerId,
        location: PointerLocation,
        target: HitTarget,
        kind: PickingEventKind,
    ) -> Self {
        Self {
            pointer,
            location,
            target,
            kind,
            propagate: true,
        }
    }

    pub const fn new_without_propagate(
        pointer: PointerId,
        location: PointerLocation,
        target: HitTarget,
        kind: PickingEventKind,
    ) -> Self {
        Self {
            pointer,
            location,
            target,
            kind,
            propagate: false,
        }
    }

    pub const fn label(&self) -> PickingEventLabel {
        self.kind.label()
    }
}
