use serde::{Deserialize, Serialize};

use crate::ui::dispatch::UiPointerId;
use crate::ui::event_ui::UiNodeId;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiPickMode {
    #[default]
    Inherit,
    Receive,
    PassThrough,
    Block,
    Transparent,
    Ignore,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiPickPolicy {
    pub pointer: UiPickMode,
    pub focus: UiPickMode,
    pub accessibility: UiPickMode,
    pub hover: UiPickMode,
    pub capture: UiPickMode,
    pub text_hit: bool,
}

impl Default for UiPickPolicy {
    fn default() -> Self {
        Self {
            pointer: UiPickMode::Inherit,
            focus: UiPickMode::Inherit,
            accessibility: UiPickMode::Inherit,
            hover: UiPickMode::Inherit,
            capture: UiPickMode::Inherit,
            text_hit: false,
        }
    }
}

impl UiPickPolicy {
    pub const fn receive() -> Self {
        Self {
            pointer: UiPickMode::Receive,
            focus: UiPickMode::Receive,
            accessibility: UiPickMode::Receive,
            hover: UiPickMode::Receive,
            capture: UiPickMode::Receive,
            text_hit: false,
        }
    }

    pub const fn ignore() -> Self {
        Self {
            pointer: UiPickMode::Ignore,
            focus: UiPickMode::Ignore,
            accessibility: UiPickMode::Ignore,
            hover: UiPickMode::Ignore,
            capture: UiPickMode::Ignore,
            text_hit: false,
        }
    }

    pub const fn with_focus(mut self, focus: UiPickMode) -> Self {
        self.focus = focus;
        self
    }

    pub const fn with_accessibility(mut self, accessibility: UiPickMode) -> Self {
        self.accessibility = accessibility;
        self
    }

    pub const fn with_text_hit(mut self, text_hit: bool) -> Self {
        self.text_hit = text_hit;
        self
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiPointerCaptureKind {
    #[default]
    Press,
    Drag,
    Programmatic,
    TextSelection,
    ScrollThumb,
    AccessibilityAction,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiPointerCapture {
    pub owner: Option<UiNodeId>,
    pub pointer_id: Option<UiPointerId>,
    pub kind: UiPointerCaptureKind,
    pub active: bool,
}
