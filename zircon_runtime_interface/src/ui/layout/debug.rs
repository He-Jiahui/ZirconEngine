use serde::{Deserialize, Serialize};

use crate::ui::event_ui::UiNodeId;

use super::{
    BoxConstraints, UiFrame, UiLayoutEngineBackend, UiLayoutEngineFallbackReason,
    UiLayoutEngineSelectionReport, UiLayoutStyleSourceRef,
};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiLayoutDebugPacket {
    pub frame_index: u64,
    pub selection_report: UiLayoutEngineSelectionReport,
    pub nodes: Vec<UiLayoutDebugNode>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiLayoutDebugNode {
    pub node_id: UiNodeId,
    pub geometry: UiFrame,
    pub constraints: BoxConstraints,
    pub engine: Option<UiLayoutEngineBackend>,
    pub fallback_reason: Option<UiLayoutEngineFallbackReason>,
    pub style_sources: Vec<UiLayoutStyleSourceRef>,
}
