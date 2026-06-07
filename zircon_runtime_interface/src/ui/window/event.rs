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

    pub const fn window_focused(metadata: UiWindowEventMetadata, focused: bool) -> Self {
        Self::new(metadata, UiWindowEventKind::Focused { focused })
    }

    pub const fn window_activation_changed(
        metadata: UiWindowEventMetadata,
        activation: UiWindowActivation,
    ) -> Self {
        Self::window_focused(metadata, activation.is_active())
    }

    pub const fn application_activation_changed(
        metadata: UiWindowEventMetadata,
        is_active: bool,
    ) -> Self {
        Self::window_focused(metadata, is_active)
    }

    pub const fn size_changed(metadata: UiWindowEventMetadata, metrics: UiWindowMetrics) -> Self {
        Self::new(metadata, UiWindowEventKind::Resized { metrics })
    }

    pub const fn os_paint(metadata: UiWindowEventMetadata) -> Self {
        Self::request_redraw(metadata, UiWindowRedrawReason::Paint)
    }

    pub const fn resizing_window(metadata: UiWindowEventMetadata) -> Self {
        Self::request_redraw(metadata, UiWindowRedrawReason::Paint)
    }

    pub const fn window_action(metadata: UiWindowEventMetadata, action: UiWindowAction) -> Self {
        Self::new(metadata, UiWindowEventKind::WindowAction { action })
    }

    pub const fn moved_window(
        metadata: UiWindowEventMetadata,
        position: UiWindowPixelPosition,
    ) -> Self {
        Self::new(metadata, UiWindowEventKind::Moved { position })
    }

    pub const fn window_close(metadata: UiWindowEventMetadata) -> Self {
        Self::new(metadata, UiWindowEventKind::CloseRequested)
    }

    pub const fn request_redraw(
        metadata: UiWindowEventMetadata,
        reason: UiWindowRedrawReason,
    ) -> Self {
        Self::new(metadata, UiWindowEventKind::RequestRedraw { reason })
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
    WindowAction {
        action: UiWindowAction,
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
            Self::Occluded { .. } | Self::Moved { .. } | Self::WindowAction { .. } => {
                UiWindowEventImpact::clean()
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UiWindowAction {
    #[default]
    ClickedNonClientArea,
    Maximize,
    Restore,
    WindowMenu,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UiWindowActivation {
    #[default]
    Activate,
    ActivateByMouse,
    Deactivate,
}

impl UiWindowActivation {
    pub const fn is_active(self) -> bool {
        matches!(self, Self::Activate | Self::ActivateByMouse)
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
