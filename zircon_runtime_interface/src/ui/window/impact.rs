use serde::{Deserialize, Serialize};

/// Declarative consequences of a neutral window event. Runtime/editor hosts may
/// map these to their own dirty bits without re-interpreting platform variants.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiWindowEventImpact {
    pub input_state_dirty: bool,
    pub layout_metrics_dirty: bool,
    pub clears_hover: bool,
    pub requests_redraw: bool,
    pub close_requested: bool,
}

impl UiWindowEventImpact {
    pub const fn clean() -> Self {
        Self {
            input_state_dirty: false,
            layout_metrics_dirty: false,
            clears_hover: false,
            requests_redraw: false,
            close_requested: false,
        }
    }

    pub const fn input_state() -> Self {
        Self {
            input_state_dirty: true,
            layout_metrics_dirty: false,
            clears_hover: false,
            requests_redraw: false,
            close_requested: false,
        }
    }

    pub const fn layout_metrics() -> Self {
        Self {
            input_state_dirty: false,
            layout_metrics_dirty: true,
            clears_hover: false,
            requests_redraw: false,
            close_requested: false,
        }
    }

    pub const fn redraw() -> Self {
        Self {
            input_state_dirty: false,
            layout_metrics_dirty: false,
            clears_hover: false,
            requests_redraw: true,
            close_requested: false,
        }
    }

    pub const fn close_requested() -> Self {
        Self {
            input_state_dirty: false,
            layout_metrics_dirty: false,
            clears_hover: false,
            requests_redraw: false,
            close_requested: true,
        }
    }

    pub const fn with_redraw(mut self) -> Self {
        self.requests_redraw = true;
        self
    }

    pub const fn with_hover_clear(mut self) -> Self {
        self.clears_hover = true;
        self
    }
}
