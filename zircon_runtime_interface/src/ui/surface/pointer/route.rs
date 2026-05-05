use serde::{Deserialize, Serialize};

use crate::ui::event_ui::UiNodeId;
use crate::ui::layout::UiPoint;
use crate::ui::surface::UiHitPath;

use super::{UiPointerActivationPhase, UiPointerButton, UiPointerEventKind};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiPointerRoute {
    pub kind: UiPointerEventKind,
    pub button: Option<UiPointerButton>,
    #[serde(default)]
    pub activation_phase: UiPointerActivationPhase,
    pub point: UiPoint,
    pub scroll_delta: f32,
    pub target: Option<UiNodeId>,
    #[serde(default)]
    pub hit_path: UiHitPath,
    pub bubbled: Vec<UiNodeId>,
    pub stacked: Vec<UiNodeId>,
    pub entered: Vec<UiNodeId>,
    pub left: Vec<UiNodeId>,
    pub captured: Option<UiNodeId>,
    #[serde(default)]
    pub pressed: Option<UiNodeId>,
    #[serde(default)]
    pub click_target: Option<UiNodeId>,
    #[serde(default)]
    pub release_inside_pressed: bool,
    pub focused: Option<UiNodeId>,
    pub fallback_to_root: bool,
    pub root_targets: Vec<UiNodeId>,
}
