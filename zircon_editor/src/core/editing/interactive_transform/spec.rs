use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PivotMode {
    Primary,
    #[default]
    Centroid,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum InteractiveTransformKind {
    Move,
    Rotate,
    Scale,
}

impl InteractiveTransformKind {
    pub(crate) const fn history_label(self) -> &'static str {
        match self {
            Self::Move => "Move scene selection",
            Self::Rotate => "Rotate scene selection",
            Self::Scale => "Scale scene selection",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum InteractiveTransformAxis {
    X,
    Y,
    Z,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum InteractiveTransformSpace {
    Global,
    Local,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct InteractiveTransformSpec {
    kind: InteractiveTransformKind,
    axis: InteractiveTransformAxis,
    space: InteractiveTransformSpace,
    snap_enabled: bool,
}

impl InteractiveTransformSpec {
    pub(crate) const fn new(
        kind: InteractiveTransformKind,
        axis: InteractiveTransformAxis,
        space: InteractiveTransformSpace,
        snap_enabled: bool,
    ) -> Self {
        Self {
            kind,
            axis,
            space,
            snap_enabled,
        }
    }

    pub(crate) const fn kind(self) -> InteractiveTransformKind {
        self.kind
    }

    pub(crate) const fn axis(self) -> InteractiveTransformAxis {
        self.axis
    }

    pub(crate) const fn space(self) -> InteractiveTransformSpace {
        self.space
    }

    pub(crate) const fn snap_enabled(self) -> bool {
        self.snap_enabled
    }
}
