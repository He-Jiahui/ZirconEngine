use serde::{Deserialize, Serialize};

use crate::ui::event_ui::UiNodeId;
use crate::ui::layout::{UiFrame, UiGeometry};

use super::{UiBrushSet, UiTextPaint};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiPaintElement {
    pub node_id: UiNodeId,
    pub geometry: UiGeometry,
    pub clip: Option<UiClipState>,
    pub z_index: i32,
    pub paint_order: u64,
    pub payload: UiPaintPayload,
    pub effects: UiPaintEffects,
    pub cache_generation: Option<u64>,
    pub debug_label: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum UiPaintPayload {
    Empty,
    Brush { brushes: UiBrushSet },
    Text { text: UiTextPaint },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiClipState {
    pub mode: UiClipMode,
    pub frame: UiFrame,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiClipMode {
    #[default]
    Scissor,
    Stencil,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiPaintEffects {
    pub opacity: f32,
    pub effects: Vec<UiDrawEffect>,
}

impl Default for UiPaintEffects {
    fn default() -> Self {
        Self {
            opacity: 1.0,
            effects: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiDrawEffect {
    PixelSnapped,
    DisabledEffect,
    NoGamma,
}
