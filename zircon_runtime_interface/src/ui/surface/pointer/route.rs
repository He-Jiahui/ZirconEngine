use serde::{Deserialize, Serialize};

use crate::ui::event_ui::UiNodeId;
use crate::ui::layout::UiPoint;

use super::{UiPointerButton, UiPointerEventKind};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiPointerRoute {
    pub kind: UiPointerEventKind,
    pub button: Option<UiPointerButton>,
    pub point: UiPoint,
    pub scroll_delta: f32,
    pub target: Option<UiNodeId>,
    pub bubbled: Vec<UiNodeId>,
    pub stacked: Vec<UiNodeId>,
    pub entered: Vec<UiNodeId>,
    pub left: Vec<UiNodeId>,
    pub captured: Option<UiNodeId>,
    pub focused: Option<UiNodeId>,
    pub fallback_to_root: bool,
    pub root_targets: Vec<UiNodeId>,
}
