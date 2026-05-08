use serde::{Deserialize, Serialize};

use crate::ui::dispatch::UiWindowId;
use crate::ui::layout::UiPoint;

use super::{UiWindowEventImpact, UiWindowEventMetadata, UiWindowMetrics, UiWindowPixelPosition};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiWindowEvent {
    pub metadata: UiWindowEventMetadata,
    pub kind: UiWindowEventKind,
}

impl UiWindowEvent {
    pub const fn new(metadata: UiWindowEventMetadata, kind: UiWindowEventKind) -> Self {
        Self { metadata, kind }
    }

    pub fn window_id(&self) -> Option<&UiWindowId> {
        (!self.metadata.window_id.0.is_empty()).then_some(&self.metadata.window_id)
    }

    pub const fn impact(&self) -> UiWindowEventImpact {
        self.kind.impact()
    }

    pub const fn is_redraw_request(&self) -> bool {
        matches!(self.kind, UiWindowEventKind::RequestRedraw { .. })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum UiWindowEventKind {
    Created {
        #[serde(default)]
        metrics: UiWindowMetrics,
    },
    CloseRequested,
    Closed,
    Destroyed,
    CursorMoved {
        position: UiPoint,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        delta: Option<UiPoint>,
    },
    CursorEntered,
    CursorLeft,
    Focused {
        focused: bool,
    },
    Occluded {
        occluded: bool,
    },
    Resized {
        metrics: UiWindowMetrics,
    },
    ScaleFactorChanged {
        scale_factor: f64,
    },
    BackendScaleFactorChanged {
        scale_factor: f64,
    },
    Moved {
        position: UiWindowPixelPosition,
    },
    RequestRedraw {
        reason: UiWindowRedrawReason,
    },
}

impl UiWindowEventKind {
    pub const fn impact(&self) -> UiWindowEventImpact {
        match self {
            Self::Created { .. } | Self::Resized { .. } => {
                UiWindowEventImpact::layout_metrics().with_redraw()
            }
            Self::ScaleFactorChanged { .. } | Self::BackendScaleFactorChanged { .. } => {
                UiWindowEventImpact::layout_metrics()
            }
            Self::CursorMoved { .. } | Self::CursorEntered | Self::Focused { .. } => {
                UiWindowEventImpact::input_state()
            }
            Self::CursorLeft => UiWindowEventImpact::input_state()
                .with_hover_clear()
                .with_redraw(),
            Self::RequestRedraw { .. } => UiWindowEventImpact::redraw(),
            Self::CloseRequested => UiWindowEventImpact::close_requested(),
            Self::Closed | Self::Destroyed => UiWindowEventImpact::input_state().with_hover_clear(),
            Self::Occluded { .. } | Self::Moved { .. } => UiWindowEventImpact::clean(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UiWindowRedrawReason {
    #[default]
    Host,
    Input,
    Animation,
    Layout,
    Paint,
    Diagnostics,
}
